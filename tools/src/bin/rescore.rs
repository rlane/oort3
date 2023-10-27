use clap::Parser;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use indicatif::{MultiProgress, ProgressBar};
use oort_proto::LeaderboardSubmission;
use oort_simulator::{scenario, simulation};
use oort_tools::ParallelCompiler;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,

    #[clap(short = 'n', long, value_parser, default_value_t = false)]
    dry_run: bool,

    #[clap(short, long, value_parser)]
    scenario: Option<String>,

    #[clap(long, value_parser, default_value_t = 10)]
    limit: usize,
}

#[derive(Serialize, Deserialize)]
struct JsonMap {
    #[serde(flatten)]
    fields: HashMap<String, serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("rescore=info"))
            .build();
    let multi = MultiProgress::new();
    indicatif_log_bridge::LogWrapper::new(multi.clone(), logger)
        .try_init()
        .unwrap();

    let args = Arguments::parse();

    let db = FirestoreDb::new(&args.project_id).await?;
    let compiler = ParallelCompiler::new(4);

    let mut scenario_names = vec![];
    if let Some(scenario) = args.scenario.as_ref() {
        scenario_names.push(scenario.clone());
    } else {
        scenario_names = scenario::list()
            .iter()
            .flat_map(|(_, v)| v.clone())
            .collect();
    }

    let mut all_docs = vec![];
    for scenario_name in &scenario_names {
        log::info!("Querying scenario {}", scenario_name);

        let docs: Vec<Document> = db
            .query_doc(
                FirestoreQueryParams::new("leaderboard".into())
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
                            "time".to_owned(),
                            FirestoreQueryDirection::Ascending,
                        ),
                        FirestoreQueryOrder::new(
                            "timestamp".to_owned(),
                            FirestoreQueryDirection::Ascending,
                        ),
                    ])
                    .with_limit(args.limit as u32),
            )
            .await?;
        all_docs.extend(docs);
    }

    let progress = multi.add(indicatif::ProgressBar::new(all_docs.len() as u64 * 10));
    let updates: Vec<(String, LeaderboardSubmission, Option<LeaderboardSubmission>)> = all_docs
        .par_iter()
        .filter_map(|doc| {
            let docid = extract_docid(&doc.name);
            if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
                log::debug!(
                    "{}/{}: Running simulations old_time={:.3} docid={}",
                    msg.scenario_name,
                    msg.username,
                    msg.time,
                    docid
                );

                let wasm = match compiler.compile(&msg.code) {
                    Ok(wasm) => wasm,
                    Err(e) => {
                        log::warn!(
                            "{}/{}: Compilation failed for docid={}: {}",
                            msg.scenario_name,
                            msg.username,
                            docid,
                            e
                        );
                        return Some((doc.name.to_string(), msg.clone(), None));
                    }
                };

                log::debug!(
                    "{}/{}: Successfully compiled",
                    msg.scenario_name,
                    msg.username
                );
                let status = run_simulations(&msg.scenario_name, wasm, &progress);
                match status {
                    Some(new_time) => {
                        if (msg.time - new_time).abs() >= 0.001 {
                            log::info!(
                                "{}/{}: Updating time from {:.3} to {:.3}",
                                msg.scenario_name,
                                msg.username,
                                msg.time,
                                new_time
                            );
                            let mut new_msg = msg.clone();
                            new_msg.time = new_time;
                            Some((doc.name.to_string(), msg.clone(), Some(new_msg)))
                        } else {
                            log::debug!(
                                "{}/{}: Time unchanged ({:.3})",
                                msg.scenario_name,
                                msg.username,
                                new_time
                            );
                            None
                        }
                    }
                    None => {
                        log::warn!(
                            "{}/{}: Simulation failed for docid={}",
                            msg.username,
                            msg.scenario_name,
                            docid,
                        );
                        Some((doc.name.to_string(), msg.clone(), None))
                    }
                }
            } else {
                None
            }
        })
        .collect();

    log::info!("Applying {} updates:", updates.len());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Scenario", "User", "Old Time", "New Time", "Docid"]);
    for (docname, old_msg, new_msg) in &updates {
        let docid = extract_docid(docname);
        table.add_row(vec![
            old_msg.scenario_name.clone(),
            old_msg.username.clone(),
            format!("{:.3}", old_msg.time),
            format!("{:.3?}", new_msg.as_ref().map(|x| x.time)),
            docid.clone(),
        ]);
    }
    println!("{table}");

    if args.dry_run {
        log::info!("Dry run, skipping");
        return Ok(());
    }

    for (docname, _old_msg, new_msg) in &updates {
        let docid = extract_docid(docname);
        if let Some(new_msg) = new_msg {
            db.update_obj("leaderboard", &docid, new_msg, None, None, None)
                .await?;
        } else {
            db.delete_by_id("leaderboard", &docid, None).await?;
        }
    }

    Ok(())
}

fn run_simulations(scenario_name: &str, wasm: Vec<u8>, progress: &ProgressBar) -> Option<f64> {
    let results: Vec<Option<f64>> = (0..10u32)
        .into_par_iter()
        .map(|seed| {
            let ret = run_simulation(scenario_name, seed, wasm.clone());
            progress.inc(1);
            ret
        })
        .collect();
    log::info!("Results: {:?}", results);
    if results.iter().any(|x| x.is_none()) {
        return None;
    }
    Some(results.iter().map(|x| x.unwrap()).sum::<f64>() / results.len() as f64)
}

fn run_simulation(scenario_name: &str, seed: u32, wasm: Vec<u8>) -> Option<f64> {
    let scenario = scenario::load(scenario_name);
    let mut codes = scenario.initial_code();
    codes[0] = simulation::Code::Wasm(wasm);
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    match sim.status() {
        scenario::Status::Victory { team: 0 } => Some(sim.score_time()),
        _ => None,
    }
}

fn extract_docid(docname: &str) -> String {
    let (_, docid) = docname.rsplit_once('/').unwrap();
    docid.to_string()
}
