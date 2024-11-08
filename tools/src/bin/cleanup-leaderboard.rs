use std::collections::HashMap;

// Remove leaderboard entries with the same username.
use clap::Parser;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::LeaderboardSubmission;

const COLLECTION_NAME: &str = "leaderboard";

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,
    #[clap(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let args = Arguments::parse();
    run(&args.project_id, args.dry_run).await
}

async fn run(project_id: &str, dry_run: bool) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;

    let docs: Vec<Document> = db
        .query_doc(FirestoreQueryParams::new(COLLECTION_NAME.into()))
        .await?;

    // scenario_name -> username -> (docid, doc)
    let mut map: HashMap<String, HashMap<String, Vec<(String, LeaderboardSubmission)>>> =
        HashMap::new();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            map.entry(msg.scenario_name.clone())
                .or_default()
                .entry(msg.username.clone())
                .or_default()
                .push((doc.name.clone(), msg));
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    for (scenario_name, user_map) in map.iter_mut() {
        for (username, entries) in user_map.iter_mut() {
            if entries.len() > 1 {
                log::info!(
                    "Removing duplicate entries for {} in {}",
                    username,
                    scenario_name
                );
                let best_docid = entries
                    .iter()
                    .min_by(|a, b| a.1.time.total_cmp(&b.1.time))
                    .as_ref()
                    .unwrap()
                    .0
                    .clone();
                for (docid, _) in entries.iter() {
                    if *docid == best_docid {
                        continue;
                    }
                    log::info!("Deleting duplicate {}", docid);
                    if !dry_run {
                        let docid = docid.rsplit_once('/').unwrap().1;
                        db.delete_by_id(COLLECTION_NAME, docid, None).await?;
                    }
                }
            }
        }
    }

    Ok(())
}
