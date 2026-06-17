#![cfg_attr(test, allow(dead_code))]

use chrono::Utc;
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use sha2::{Digest, Sha256};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;

use oort_proto::{ShortcodeUpload, TournamentCompetitor, TournamentResults, TournamentSubmission};
use oort_simulator::{scenario, simulation};
use oort_tools::AI;
use oort_tools::process_pool::ProcessPool;
use rand::RngExt;

use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use std::default::Default;
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum WorkerTask {
    RegisterAIs {
        ais: Vec<AI>,
    },
    RunSimulation {
        scenario_name: String,
        seed: u32,
        ai_indices: Vec<usize>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
enum WorkerResponse {
    Registered,
    SimulationResult(Result<Outcomes, String>),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheEntry {
    player0_hash: String,
    player1_hash: String,
    start_seed: u32,
    num_seeds: u32,
    wins: u32,
    losses: u32,
    draws: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct IncrementalCache {
    entries: Vec<CacheEntry>,
}

fn get_code_hash(source_code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_code.as_bytes());
    hex::encode(hasher.finalize())
}

fn get_start_seed(hash0: &str, hash1: &str) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", hash0, hash1).as_bytes());
    let result = hasher.finalize();
    u32::from_be_bytes(result[0..4].try_into().unwrap())
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn get_pair_outcomes(
    i: usize,
    j: usize,
    hash_i: &str,
    hash_j: &str,
    target_rounds: i32,
    cache: Option<&mut IncrementalCache>,
    to_simulate: &mut Vec<(i32, Vec<usize>, u32)>,
    sim_matchups_info: &mut Vec<(usize, usize, i32)>,
    cached_outcomes: &mut Vec<(i32, Vec<usize>, u32, Result<Outcomes, String>)>,
    seeds: &[u32],
) {
    if cache.is_none() {
        for round in 0..target_rounds {
            to_simulate.push((round, vec![i, j], seeds[round as usize]));
        }
        return;
    }

    let mut cache_hit = false;
    if let Some(entry) = cache.as_ref().and_then(|c| {
        c.entries.iter().find(|e| {
            e.player0_hash == *hash_i
                && e.player1_hash == *hash_j
                && e.num_seeds == target_rounds as u32
        })
    }) {
        cache_hit = true;
        let mut w = entry.wins as usize;
        let mut l = entry.losses as usize;
        let mut d = entry.draws as usize;
        let start_seed = entry.start_seed;
        for round in 0..target_rounds {
            let outcome = if w > 0 {
                w -= 1;
                Outcomes::WIN
            } else if l > 0 {
                l -= 1;
                Outcomes::LOSS
            } else {
                d = d.saturating_sub(1);
                Outcomes::DRAW
            };
            cached_outcomes.push((round, vec![i, j], start_seed.wrapping_add(round as u32), Ok(outcome)));
        }
    }

    if !cache_hit {
        let start_seed = get_start_seed(hash_i, hash_j);
        for round in 0..target_rounds {
            to_simulate.push((round, vec![i, j], start_seed.wrapping_add(round as u32)));
            sim_matchups_info.push((i, j, target_rounds));
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_cache_from_simulations(
    cache: &mut IncrementalCache,
    sim_outcomes: &[(i32, Vec<usize>, u32, Result<Outcomes, String>)],
    sim_matchups_info: &[(usize, usize, i32)],
    ai_hashes: &[String],
) {
    let mut pair_results: HashMap<(usize, usize, i32), (u32, u32, u32)> = HashMap::new();
    for (idx, (_, _, _, outcome_res)) in sim_outcomes.iter().enumerate() {
        let (i, j, target_rounds) = sim_matchups_info[idx];
        let counts = pair_results.entry((i, j, target_rounds)).or_insert((0, 0, 0));
        if let Ok(outcome) = outcome_res {
            match outcome {
                Outcomes::WIN => counts.0 += 1,
                Outcomes::LOSS => counts.1 += 1,
                Outcomes::DRAW => counts.2 += 1,
            }
        } else {
            counts.2 += 1;
        }
    }

    for ((i, j, target_rounds), (wins, losses, draws)) in pair_results {
        let hash_i = &ai_hashes[i];
        let hash_j = &ai_hashes[j];
        let start_seed = get_start_seed(hash_i, hash_j);
        
        if let Some(entry) = cache.entries.iter_mut().find(|e| e.player0_hash == *hash_i && e.player1_hash == *hash_j) {
            entry.num_seeds = target_rounds as u32;
            entry.start_seed = start_seed;
            entry.wins = wins;
            entry.losses = losses;
            entry.draws = draws;
        } else {
            cache.entries.push(CacheEntry {
                player0_hash: hash_i.clone(),
                player1_hash: hash_j.clone(),
                start_seed,
                num_seeds: target_rounds as u32,
                wins,
                losses,
                draws,
            });
        }
    }
}

#[allow(clippy::type_complexity)]
fn run_simulations_parallel(
    pool: &ProcessPool<WorkerTask, WorkerResponse>,
    scenario_name: &str,
    ais: &[AI],
    matchups: Vec<(i32, Vec<usize>, u32)>, // (round, indices, seed)
    progress: &indicatif::ProgressBar,
) -> Vec<(i32, Vec<usize>, u32, Result<Outcomes, String>)> {
    let ai_data: Vec<AI> = ais.to_vec();

    // 1. Register AIs with all workers
    let responses = pool.broadcast(WorkerTask::RegisterAIs { ais: ai_data });
    for (idx, response) in responses.into_iter().enumerate() {
        match response {
            Ok(WorkerResponse::Registered) => {}
            Ok(WorkerResponse::Error(err)) => {
                log::error!("Worker {} failed to register AIs: {}", idx, err);
            }
            Err(err) => {
                log::error!("Worker {} coordinator reported error during registration: {}", idx, err);
            }
            _ => panic!("Unexpected response during registration"),
        }
    }

    // 2. Execute matchups
    let requests: Vec<WorkerTask> = matchups
        .iter()
        .map(|(_, ai_indices, seed)| WorkerTask::RunSimulation {
            scenario_name: scenario_name.to_string(),
            seed: *seed,
            ai_indices: ai_indices.clone(),
        })
        .collect();

    let responses = pool.execute(&requests, || progress.inc(1));

    // Convert responses into final results
    let mut results = Vec::with_capacity(matchups.len());
    for (idx, response) in responses.into_iter().enumerate() {
        let (round, ai_indices, seed) = &matchups[idx];
        let outcome_res = match response {
            Ok(WorkerResponse::SimulationResult(res)) => res,
            Ok(WorkerResponse::Registered) => Err("Received unexpected Registered response".to_string()),
            Ok(WorkerResponse::Error(err)) => Err(err),
            Err(err) => Err(err),
        };
        results.push((
            *round,
            ai_indices.clone(),
            *seed,
            outcome_res,
        ));
    }
    results
}



#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Run {
        scenario: String,
        usernames: Vec<String>,

        #[clap(short, long, default_value_t = 100)]
        rounds: i32,

        #[clap(short, long)]
        dry_run: bool,
    },
    RunUnofficial {
        scenario: String,
        shortcodes: Vec<String>,

        #[clap(short, long, default_value_t = 100)]
        rounds: i32,

        #[clap(short, long)]
        dev: bool,

        #[clap(long, default_value = "/tmp/oort-wasm-cache")]
        wasm_cache: Option<PathBuf>,
    },

    Fetch {
        scenario: String,
        out_dir: String,
    },
    Write {
        scenario: String,
        username: String,
        path: String,
    },
}

#[derive(Debug, Clone)]
struct Entrant {
    username: String,
    source_code: String,
}

#[cfg(not(test))]
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("tournament=info"))
        .init();

    let args = Arguments::parse();
    let needs_workers = matches!(
        &args.cmd,
        SubCommand::Run { .. } | SubCommand::RunUnofficial { .. }
    );

    let pool = if needs_workers {
        let mut registered_ais = Vec::new();
        Some(ProcessPool::new(move |req: WorkerTask| -> WorkerResponse {
            match req {
                WorkerTask::RegisterAIs { ais } => {
                    registered_ais = ais;
                    WorkerResponse::Registered
                }
                WorkerTask::RunSimulation { scenario_name, seed, ai_indices } => {
                    let ais_for_sim: Vec<&AI> = ai_indices.iter().map(|&idx| &registered_ais[idx]).collect();
                    let res = run_simulation(&scenario_name, seed, &ais_for_sim);
                    WorkerResponse::SimulationResult(res)
                }
            }
        }))
    } else {
        None
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let res = rt.block_on(async {
        match args.cmd {
            SubCommand::Run {
                ref scenario,
                ref usernames,
                rounds,
                dry_run,
            } => {
                let pool_ref = pool.as_ref().expect("Process pool must be initialized for Run");
                cmd_run(pool_ref, &args.project_id, scenario, usernames, rounds, dry_run).await
            }
            SubCommand::RunUnofficial {
                ref scenario,
                ref shortcodes,
                rounds,
                dev,
                ref wasm_cache,
            } => {
                let pool_ref = pool.as_ref().expect("Process pool must be initialized for RunUnofficial");
                cmd_run_unofficial(pool_ref, scenario, shortcodes, rounds, dev, wasm_cache.clone()).await
            }
            SubCommand::Fetch { ref scenario, ref out_dir } => {
                cmd_fetch(&args.project_id, scenario, out_dir).await
            }
            SubCommand::Write {
                ref scenario,
                ref username,
                ref path,
            } => cmd_write(&args.project_id, scenario, username, path).await,
        }
    });

    drop(pool);

    res
}

#[cfg(test)]
fn main() -> anyhow::Result<()> {
    tests::run_all_tests()
}


async fn cmd_run_unofficial(
    pool: &ProcessPool<WorkerTask, WorkerResponse>,
    scenario_name: &str,
    shortcodes: &[String],
    rounds: i32,
    dev: bool,
    wasm_cache: Option<PathBuf>,
) -> anyhow::Result<()> {
    scenario::load_safe(scenario_name).expect("Unknown scenario");

    let http = reqwest::Client::new();
    let ais = oort_tools::fetch_and_compile_multiple(&http, shortcodes, dev, wasm_cache.as_deref())
        .await?;
    let ai_hashes: Vec<String> = ais.iter().map(|ai| get_code_hash(&ai.source_code)).collect();

    log::info!("Running tournament");
    let results = run_tournament(pool, scenario_name, &ais, rounds, None, &ai_hashes);

    display_results(&results);

    Ok(())
}

async fn cmd_run(
    pool: &ProcessPool<WorkerTask, WorkerResponse>,
    project_id: &str,
    scenario_name: &str,
    usernames: &[String],
    rounds: i32,
    dry_run: bool,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    scenario::load_safe(scenario_name).expect("Unknown scenario");

    let mut compiler = oort_compiler::Compiler::new();
    let entrants = get_entrants(&db, scenario_name, usernames).await?;
    let results: Vec<anyhow::Result<AI>> = entrants
        .iter()
        .map(|entrant| {
            log::info!("Compiling {:?}", entrant.username);
            let compiled_code = compiler.compile(&entrant.source_code)?;
            let compiled_code = oort_simulator::vm::precompile(&compiled_code).unwrap();
            Ok(AI {
                name: entrant.username.clone(),
                source_code: entrant.source_code.clone(),
                compiled_code,
            })
        })
        .collect();
    let ais: Vec<AI> = results.into_iter().collect::<anyhow::Result<Vec<AI>>>()?;

    let ai_hashes: Vec<String> = ais.iter().map(|ai| get_code_hash(&ai.source_code)).collect();
    let current_hashes_set: HashSet<String> = ai_hashes.iter().cloned().collect();

    log::info!("Reading incremental cache from Firestore");
    let cache_doc_id = scenario_name.to_string();
    let mut cache: IncrementalCache = match db.get_obj("tournament_incremental_cache", &cache_doc_id).await {
        Ok(c) => c,
        Err(_) => {
            log::info!("No existing cache found. Initializing new cache.");
            IncrementalCache::default()
        }
    };

    let initial_entry_count = cache.entries.len();
    cache.entries.retain(|entry| {
        current_hashes_set.contains(&entry.player0_hash) && current_hashes_set.contains(&entry.player1_hash)
    });
    log::info!(
        "Retained {}/{} cache entries based on current AIs",
        cache.entries.len(),
        initial_entry_count
    );

    log::info!("Running tournament");
    let results = run_tournament(pool, scenario_name, &ais, rounds, Some(&mut cache), &ai_hashes);

    display_results(&results);

    if !dry_run {
        upload_results(&db, project_id, &entrants, &results).await?;
        
        log::info!("Writing updated incremental cache to Firestore");
        db.update_obj::<_, (), _>("tournament_incremental_cache", cache_doc_id, &cache, None, None, None)
            .await?;
    }

    Ok(())
}



fn run_tournament(
    pool: &ProcessPool<WorkerTask, WorkerResponse>,
    scenario_name: &str,
    ais: &[AI],
    rounds: i32,
    mut cache: Option<&mut IncrementalCache>,
    ai_hashes: &[String],
) -> TournamentResults {
    let seeds: Vec<u32> = (0..rounds).map(|_| rand::rng().random()).collect();
    let config = Glicko2Config::new();
    let mut pairings: HashMap<(String, String), f64> = HashMap::new();
    let mut ratings: Vec<Glicko2Rating> = Vec::new();
    ratings.resize_with(ais.len(), Default::default);

    let mut outcomes = Vec::new();
    let mut to_simulate = Vec::new();
    let mut cached_outcomes = Vec::new();
    let mut sim_matchups_info = Vec::new();

    for i in 0..ais.len() {
        for j in 0..ais.len() {
            if i == j {
                continue;
            }
            let hash_i = &ai_hashes[i];
            let hash_j = &ai_hashes[j];
            get_pair_outcomes(
                i,
                j,
                hash_i,
                hash_j,
                rounds,
                cache.as_deref_mut(),
                &mut to_simulate,
                &mut sim_matchups_info,
                &mut cached_outcomes,
                &seeds,
            );
        }
    }

    let progress = indicatif::ProgressBar::new(to_simulate.len() as u64);
    progress.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len} Elapsed: {elapsed_precise} ETA: {eta_precise}")
            .unwrap(),
    );

    let sim_outcomes = if !to_simulate.is_empty() {
        run_simulations_parallel(pool, scenario_name, ais, to_simulate, &progress)
    } else {
        Vec::new()
    };
    progress.finish_and_clear();

    if let Some(ref mut c) = cache {
        update_cache_from_simulations(c, &sim_outcomes, &sim_matchups_info, ai_hashes);
    }

    outcomes.extend(cached_outcomes);
    outcomes.extend(sim_outcomes);
    outcomes.sort_by_key(|x| x.0);

    let mut crashes = Vec::new();
    for (round, indices, seed, outcome_res) in outcomes {
        let outcome = match outcome_res {
            Ok(outcome) => outcome,
            Err(_) => {
                crashes.push(oort_proto::TournamentCrash {
                    seed,
                    ais: indices.iter().map(|&idx| ais[idx].name.clone()).collect(),
                });
                Outcomes::DRAW
            }
        };
        let i0 = indices[0];
        let i1 = indices[1];
        log::debug!(
            "{} vs {} seed {}: {:?}",
            ais[i0].name,
            ais[i1].name,
            round,
            outcome
        );
        let (r0, r1) = glicko2(&ratings[i0], &ratings[i1], &outcome, &config);
        ratings[i0] = r0;
        ratings[i1] = r1;

        let increment = 1.0 / (2.0 * rounds as f64);
        if outcome == Outcomes::WIN {
            *pairings
                .entry((ais[i0].name.clone(), ais[i1].name.clone()))
                .or_default() += increment;
        } else if outcome == Outcomes::LOSS {
            *pairings
                .entry((ais[i1].name.clone(), ais[i0].name.clone()))
                .or_default() += increment;
        }
    }

    let mut competitors: Vec<_> = ais
        .iter()
        .enumerate()
        .map(|(i, x)| TournamentCompetitor {
            username: x.name.clone(),
            shortcode: "".to_string(),
            rating: ratings[i].rating,
        })
        .collect();
    competitors.sort_by_key(|c| (-c.rating * 1e6) as i64);

    let mut win_matrix: Vec<f64> = vec![];
    for competitor in &competitors {
        for other_competitor in &competitors {
            win_matrix.push(
                pairings
                    .get(&(
                        competitor.username.clone(),
                        other_competitor.username.clone(),
                    ))
                    .copied()
                    .unwrap_or_default(),
            );
        }
    }

    TournamentResults {
        scenario_name: scenario_name.to_string(),
        competitors,
        win_matrix,
        crashes,
    }
}

fn run_simulation(scenario_name: &str, seed: u32, ais: &[&AI]) -> Result<Outcomes, String> {
    let f = move || {
        let codes: Vec<_> = ais.iter().map(|x| x.compiled_code.clone()).collect();
        let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
        while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
            sim.step();
        }
        match sim.status() {
            scenario::Status::Victory { team: 0 } => Outcomes::WIN,
            scenario::Status::Victory { team: 1 } => Outcomes::LOSS,
            scenario::Status::Draw => Outcomes::DRAW,
            _ => unreachable!(),
        }
    };
    match ::std::panic::catch_unwind(f) {
        Ok(x) => Ok(x),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            log::error!("Simulation panicked: {}", msg);
            Err(msg)
        }
    }
}

fn display_results(results: &TournamentResults) {
    if !results.crashes.is_empty() {
        let num_crashes = results.crashes.len();
        let displayed_crashes = std::cmp::min(10, num_crashes);
        println!("Crashes ({displayed_crashes} of {num_crashes}):");
        for crash in results.crashes.iter().take(displayed_crashes) {
            println!("  Seed: {}, AIs: {:?}", crash.seed, crash.ais);
        }
        println!();
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Rating"]);
    for competitor in &results.competitors {
        table.add_row(vec![
            competitor.username.clone(),
            format!("{:.0}", competitor.rating),
        ]);
    }
    println!("Scenario: {}", results.scenario_name);
    println!("{table}");
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let mut header: Vec<String> = results
        .competitors
        .iter()
        .map(|x| x.username.clone())
        .collect();
    header.insert(0, "Winner / Loser".to_owned());
    table.set_header(header);
    let mut index = 0;
    for name0 in results.competitors.iter().map(|x| &x.username) {
        let mut row = vec![name0.clone()];
        for name1 in results.competitors.iter().map(|x| &x.username) {
            if name0 == name1 {
                row.push("".to_owned());
                index += 1;
                continue;
            }
            let frac = results.win_matrix[index];
            row.push(format!("{}", (frac * 100.0).round()));
            index += 1;
        }
        table.add_row(row);
    }
    println!("{table}");
}

async fn upload_results(
    db: &FirestoreDb,
    project_id: &str,
    entrants: &[Entrant],
    results: &TournamentResults,
) -> anyhow::Result<()> {
    log::info!("Uploading to database...");

    let tournament_id = format!(
        "{}.{}.{}",
        results.scenario_name,
        Utc::now().format("%Y%m%d"),
        rand::rng().random_range(0..10000)
    );

    let mut results = results.clone();
    for competitor in results.competitors.iter_mut() {
        let entrant = entrants
            .iter()
            .find(|x| x.username == competitor.username)
            .unwrap();
        let obj = ShortcodeUpload {
            userid: "".to_string(), // TODO
            username: competitor.username.clone(),
            timestamp: Utc::now(),
            code: entrant.source_code.clone(),
        };
        let shortcode = format!("{tournament_id}.{}", competitor.username);
        db.create_obj::<_, (), _>("shortcode", Some(&shortcode), &obj, None)
            .await?;
        competitor.shortcode = shortcode;
    }
    db.create_obj::<_, (), _>("tournament_results", Some(&tournament_id), &results, None)
        .await?;
    println!();
    if project_id == "oort-dev" {
        println!("Uploaded to http://localhost:8080/tournament/{tournament_id}");
    } else {
        println!("Uploaded to https://oort.rs/tournament/{tournament_id}");
    }
    Ok(())
}

async fn get_entrants(
    db: &FirestoreDb,
    scenario_name: &str,
    usernames: &[String],
) -> anyhow::Result<Vec<Entrant>> {
    let msgs: Vec<TournamentSubmission> = db
        .query_obj(
            FirestoreQueryParams::new("tournament".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(
                        vec![FirestoreQueryFilter::Compare(Some(
                            FirestoreQueryFilterCompare::Equal(
                                "scenario_name".into(),
                                scenario_name.into(),
                            ),
                        ))],
                        FirestoreQueryFilterCompositeOperator::And,
                    ),
                ))
                .with_order_by(vec![
                    FirestoreQueryOrder::new(
                        "username".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ]),
        )
        .await?;

    let mut map: HashMap<String, TournamentSubmission> = HashMap::new();
    for msg in msgs {
        if usernames.is_empty() || usernames.contains(&msg.username) {
            map.insert(msg.username.clone(), msg);
        }
    }

    let mut entrants = vec![];
    for msg in map.into_values() {
        entrants.push(Entrant {
            username: msg.username,
            source_code: msg.code,
        });
    }
    entrants.sort_by_key(|x| x.username.clone());
    Ok(entrants)
}

async fn cmd_fetch(project_id: &str, scenario_name: &str, out_dir: &str) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    let entrants = get_entrants(&db, scenario_name, &[]).await?;
    std::fs::create_dir_all(out_dir).unwrap();
    for entrant in entrants {
        let filename = format!("{}/{}.rs", &out_dir, entrant.username);
        std::fs::write(&filename, &entrant.source_code).unwrap();
        print!("{filename} ");
    }
    println!();

    Ok(())
}

async fn cmd_write(
    project_id: &str,
    scenario_name: &str,
    username: &str,
    path: &str,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    let userid = format!("admin-{username}");
    let docid = format!("{scenario_name}.{userid}");
    let code = std::fs::read_to_string(path)?;
    let msg = TournamentSubmission {
        userid,
        username: username.to_owned(),
        timestamp: Utc::now(),
        scenario_name: scenario_name.to_owned(),
        code,
    };
    db.update_obj::<_, (), _>("tournament", docid, &msg, None, None, None)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_run_simulation(_scenario_name: &str, _seed: u32, ais: &[&AI]) -> Result<Outcomes, String> {
        let name0 = &ais[0].name;
        let name1 = &ais[1].name;

        if (name0 == "bot4" && name1 == "bot5") || (name0 == "bot5" && name1 == "bot4") {
            return Err("mock panic".to_string());
        }

        let outcome = if name0 == "bot0" && name1 == "bot5" {
            Outcomes::WIN
        } else if name0 == "bot5" && name1 == "bot0" {
            Outcomes::LOSS
        } else if name0 < name1 {
            Outcomes::WIN
        } else {
            Outcomes::LOSS
        };
        Ok(outcome)
    }



    fn test_round_robin_tournament(pool: &ProcessPool<WorkerTask, WorkerResponse>, ais: &[AI]) {
        let ai_hashes: Vec<String> = ais.iter().map(|ai| get_code_hash(&ai.source_code)).collect();
        let results = run_tournament(pool, "fighter_duel", ais, 3, None, &ai_hashes);

        assert_eq!(results.competitors.len(), 6);
        assert_eq!(results.win_matrix.len(), 36);
        for competitor in &results.competitors {
            assert!(competitor.rating > 0.0);
        }

        // We expect bot4 vs bot5 matches to crash.
        // 3 rounds * 2 directions = 6 crashes.
        assert_eq!(results.crashes.len(), 6);
        for crash in &results.crashes {
            assert!(crash.ais.contains(&"bot4".to_string()));
            assert!(crash.ais.contains(&"bot5".to_string()));
        }

        let ranking: Vec<String> = results.competitors.iter().map(|c| c.username.clone()).collect();
        assert_eq!(ranking, vec!["bot0", "bot1", "bot2", "bot3", "bot5", "bot4"]);
    }

    fn test_incremental_cache(pool: &ProcessPool<WorkerTask, WorkerResponse>, ais: &[AI]) {
        let ai_hashes: Vec<String> = ais.iter().map(|ai| get_code_hash(&ai.source_code)).collect();
        let mut cache = IncrementalCache::default();

        // 1. First run: cache is empty, so it will simulate all matchups
        let _results1 = run_tournament(pool, "fighter_duel", ais, 3, Some(&mut cache), &ai_hashes);
        assert_eq!(cache.entries.len(), 30); // 6 * 5 = 30 pairings
        for entry in &cache.entries {
            assert_eq!(entry.num_seeds, 3);
            assert_eq!(entry.wins + entry.losses + entry.draws, 3);
        }

        // 2. Second run: cache is fully populated, so it should reuse the cached results
        // We modify a cache entry and check if the results match the modified entry
        let target_hash0 = &ai_hashes[0];
        let target_hash1 = &ai_hashes[5];
        if let Some(entry) = cache.entries.iter_mut().find(|e| e.player0_hash == *target_hash0 && e.player1_hash == *target_hash1) {
            entry.wins = 100;
            entry.losses = 0;
            entry.draws = 0;
            entry.num_seeds = 3;
        }

        let results2 = run_tournament(pool, "fighter_duel", ais, 3, Some(&mut cache), &ai_hashes);
        
        let idx0 = results2.competitors.iter().position(|c| c.username == "bot0").unwrap();
        let idx5 = results2.competitors.iter().position(|c| c.username == "bot5").unwrap();
        let win_rate = results2.win_matrix[idx0 * 6 + idx5];
        assert!((win_rate - 1.0).abs() < 1e-9);
    }

    pub fn run_all_tests() -> anyhow::Result<()> {
        let names = vec![
            "bot0".to_string(),
            "bot1".to_string(),
            "bot2".to_string(),
            "bot3".to_string(),
            "bot4".to_string(),
            "bot5".to_string(),
        ];

        let ais: Vec<AI> = names
            .into_iter()
            .map(|name| AI {
                name: name.clone(),
                source_code: name,
                compiled_code: oort_simulator::simulation::Code::None,
            })
            .collect();

        // Initialize the ProcessPool while single-threaded (before any other threads are spawned)
        let mut registered_ais = Vec::new();
        let pool = ProcessPool::new(move |req: WorkerTask| -> WorkerResponse {
            match req {
                WorkerTask::RegisterAIs { ais } => {
                    registered_ais = ais;
                    WorkerResponse::Registered
                }
                WorkerTask::RunSimulation { scenario_name, seed, ai_indices } => {
                    let ais_for_sim: Vec<&AI> = ai_indices.iter().map(|&idx| &registered_ais[idx]).collect();
                    let res = mock_run_simulation(&scenario_name, seed, &ais_for_sim);
                    WorkerResponse::SimulationResult(res)
                }
            }
        });

        println!("Running test_round_robin_tournament...");
        test_round_robin_tournament(&pool, &ais);
        println!("test_round_robin_tournament passed.");

        println!("Running test_incremental_cache...");
        test_incremental_cache(&pool, &ais);
        println!("test_incremental_cache passed.");

        Ok(())
    }
}
