use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::LeaderboardSubmission;
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;

const TOP_N: u32 = 10;

pub async fn rescore(dry_run: bool) -> anyhow::Result<()> {
    let db = FirestoreDb::new(&crate::project_id()).await?;
    let http = reqwest::Client::new();
    let current_version = oort_version::version();

    let scenario_names: Vec<String> = scenario::list()
        .iter()
        .flat_map(|(_, v)| v.clone())
        .collect();

    for scenario_name in &scenario_names {
        log::info!("Processing scenario {}", scenario_name);
        let mut updates: Vec<(String, LeaderboardSubmission, Option<LeaderboardSubmission>)> =
            Vec::new();

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
                    .with_limit(TOP_N),
            )
            .await?;

        for doc in docs {
            let docid = extract_docid(&doc.name);
            if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(&doc) {
                if let Some(ref rescored_version) = msg.rescored_version {
                    if *rescored_version == current_version {
                        log::info!(
                            "Skipping rescore for userid={} scenario_name={} docid={}",
                            msg.userid,
                            msg.scenario_name,
                            docid
                        );
                        continue;
                    }
                }

                log::info!(
                    "Running simulations for username={} scenario={} old_time={} docid={}",
                    msg.username,
                    msg.scenario_name,
                    msg.time,
                    docid
                );

                let code = compile(&http, &docid, &msg.code).await;

                let wasm = match code {
                    Ok(wasm) => wasm,
                    Err(e) => {
                        log::warn!(
                            "Compilation failed for userid={} scenario_name={} docid={}: {}",
                            msg.username,
                            msg.scenario_name,
                            docid,
                            e
                        );
                        continue;
                    }
                };

                log::info!("Successfully compiled to WASM");
                let status = run_simulations(&msg.scenario_name, &wasm);
                match status {
                    Some(new_time) => {
                        if (msg.time - new_time).abs() >= 0.001 {
                            log::info!("Updating time from {} to {}", msg.time, new_time);
                        } else {
                            log::info!("Time unchanged, {}", new_time);
                        }
                        let mut new_msg = msg.clone();
                        new_msg.time = new_time;
                        new_msg.rescored_version = Some(current_version.clone());
                        updates.push((doc.name.to_string(), msg.clone(), Some(new_msg)));
                    }
                    None => {
                        log::warn!(
                            "Simulation failed for userid={} scenario_name={} docid={}",
                            msg.username,
                            msg.scenario_name,
                            docid,
                        );
                        updates.push((doc.name.to_string(), msg.clone(), None));
                    }
                }
            }
        }

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

        if dry_run {
            log::info!("Dry run, skipping database update");
        } else {
            for (docname, _old_msg, new_msg) in &updates {
                let docid = extract_docid(docname);
                if let Some(new_msg) = new_msg {
                    db.update_obj("leaderboard", &docid, new_msg, None, None, None)
                        .await?;
                } else {
                    db.delete_by_id("leaderboard", &docid, None).await?;
                }
            }
        }
    }

    Ok(())
}

async fn compile(http: &reqwest::Client, name: &str, source_code: &str) -> anyhow::Result<Code> {
    let compiler_url =
        std::env::var("COMPILER_URL").unwrap_or_else(|_| "https://compiler.oort.rs".to_string());
    log::info!("Using compiler at {}", compiler_url);

    let response = http
        .post(&format!("{compiler_url}/compile"))
        .body(source_code.to_string())
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to compile {:?}: {:?}", name, response.text().await?);
    }

    let compiled_code = response.bytes().await?.to_vec();
    Ok(oort_simulator::vm::precompile(&compiled_code).unwrap())
}

fn run_simulations(scenario_name: &str, code: &Code) -> Option<f64> {
    let results: Vec<Option<f64>> = (0..10u32)
        .into_par_iter()
        .map(|seed| run_simulation(scenario_name, seed, code.clone()))
        .collect();
    log::info!("Results: {:?}", results);
    if results.iter().any(|x| x.is_none()) {
        return None;
    }
    Some(results.iter().map(|x| x.unwrap()).sum::<f64>() / results.len() as f64)
}

fn run_simulation(scenario_name: &str, seed: u32, code: Code) -> Option<f64> {
    let scenario = scenario::load(scenario_name);
    let mut codes = scenario.initial_code();
    codes[0] = code;
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
