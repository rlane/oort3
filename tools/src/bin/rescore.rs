use clap::Parser;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::LeaderboardSubmission;
use oort_simulator::{scenario, simulation};
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("rescore=info"))
        .init();

    let args = Arguments::parse();

    let db = FirestoreDb::new(&args.project_id).await?;
    let http_client = reqwest::Client::new();
    let mut updates: Vec<(String, LeaderboardSubmission, Option<LeaderboardSubmission>)> =
        Vec::new();

    let mut scenario_names = vec![];
    if let Some(scenario) = args.scenario.as_ref() {
        scenario_names.push(scenario.clone());
    } else {
        scenario_names = scenario::list();
    }

    for scenario_name in &scenario_names {
        log::info!("Processing scenario {}", scenario_name);

        let docs: Vec<Document> = db
            .query_doc(
                FirestoreQueryParams::new("leaderboard".into())
                    .with_filter(FirestoreQueryFilter::Composite(
                        FirestoreQueryFilterComposite::new(vec![FirestoreQueryFilter::Compare(
                            Some(FirestoreQueryFilterCompare::Equal(
                                "scenario_name".into(),
                                scenario_name.into(),
                            )),
                        )]),
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

        for doc in docs {
            let docid = extract_docid(&doc.name);
            if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(&doc) {
                log::info!(
                    "Running simulations for username={} scenario={} old_time={} docid={}",
                    msg.username,
                    msg.scenario_name,
                    msg.time,
                    docid
                );

                if let Some(wasm) = compile(&http_client, docid.clone(), msg.code.clone()).await {
                    log::info!("Successfully compiled to WASM");
                    let status = run_simulations(&msg.scenario_name, wasm);
                    match status {
                        Some(new_time) => {
                            if msg.time != new_time {
                                log::info!("Updating time from {} to {}", msg.time, new_time);
                                let mut new_msg = msg.clone();
                                new_msg.time = new_time;
                                updates.push((doc.name.to_string(), msg.clone(), Some(new_msg)));
                            } else {
                                log::info!("Time unchanged");
                            }
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
                } else {
                    log::warn!(
                        "Compilation failed for userid={} scenario_name={} docid={}",
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
            format!("{:.2}", old_msg.time),
            format!("{:.2?}", new_msg.as_ref().map(|x| x.time)),
            docid.clone(),
        ]);
    }
    println!("{}", table);

    if args.dry_run {
        log::info!("Dry run, skipping");
        return Ok(());
    }

    for (docname, _old_msg, new_msg) in &updates {
        let docid = extract_docid(docname);
        if let Some(new_msg) = new_msg {
            db.update_obj("leaderboard", &docid, new_msg, None).await?;
        } else {
            db.delete_by_id("leaderboard", &docid).await?;
        }
    }

    Ok(())
}

async fn compile(client: &reqwest::Client, docid: String, code: String) -> Option<Vec<u8>> {
    let url = "http://localhost:8081/compile";
    let result = client.post(url).body(code).send().await;
    let response = result.unwrap().error_for_status();
    match response {
        Ok(response) => Some(response.bytes().await.unwrap().as_ref().into()),
        Err(e) => {
            log::warn!("Failed to compile {:?}: {}", docid, e);
            None
        }
    }
}

fn run_simulations(scenario_name: &str, wasm: Vec<u8>) -> Option<f64> {
    let results: Vec<Option<f64>> = (0..10u32)
        .into_par_iter()
        .map(|seed| run_simulation(scenario_name, seed, wasm.clone()))
        .collect();
    log::info!("Results: {:?}", results);
    if results.iter().any(|x| x.is_none()) {
        return None;
    }
    Some(results.iter().map(|x| x.unwrap()).sum::<f64>() as f64 / results.len() as f64)
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
        scenario::Status::Victory { team: 0 } => Some(sim.time()),
        _ => None,
    }
}

fn extract_docid(docname: &str) -> String {
    let (_, docid) = docname.rsplit_once('/').unwrap();
    docid.to_string()
}
