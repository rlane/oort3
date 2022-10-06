use clap::Parser;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_telemetry_proto::{Telemetry, TelemetryMsg};

const COLLECTION_NAME: &str = "telemetry";

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,

    #[clap(short = 'n', long, value_parser)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let args = Arguments::parse();
    run(&args).await
}

async fn run(args: &Arguments) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(&args.project_id).await?;

    let mut docs: Vec<Document> = db
        .query_doc(FirestoreQueryParams::new(COLLECTION_NAME.into()).clone())
        .await?;
    docs.sort_by_key(|doc| doc.create_time.clone().map(|x| x.seconds).unwrap_or(0));
    for doc in &docs {
        if let Ok(original_msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let mut msg = original_msg.clone();
            match &mut msg.payload {
                Telemetry::StartScenario { .. } => {}
                Telemetry::FinishScenario { .. } => {}
                Telemetry::Crash { .. } => {}
                Telemetry::SubmitToTournament { .. } => {}
            }
            if msg != original_msg {
                let docid = doc.name.rsplit_once('/').unwrap().1.to_string();
                log::info!("Updating doc {:?}", docid);
                db.update_obj(COLLECTION_NAME, &docid, &msg, None).await?;
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    Ok(())
}

pub fn generate_username(userid: &str) -> String {
    let mut rng: rand_chacha::ChaCha8Rng = rand_seeder::Seeder::from(userid).make_rng();
    petname::Petnames::default().generate(&mut rng, 2, "-")
}
