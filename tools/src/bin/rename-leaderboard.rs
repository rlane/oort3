use clap::Parser;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::LeaderboardSubmission;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,
    src_scenario: String,
    dst_scenario: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let args = Arguments::parse();
    run(&args.project_id, &args.src_scenario, &args.dst_scenario).await
}

async fn run(
    project_id: &str,
    src_scenario_name: &str,
    dst_scenario_name: &str,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new("leaderboard".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(vec![FirestoreQueryFilter::Compare(Some(
                        FirestoreQueryFilterCompare::Equal(
                            "scenario_name".into(),
                            src_scenario_name.into(),
                        ),
                    ))]),
                ))
                .with_order_by(vec![
                    FirestoreQueryOrder::new("time".to_owned(), FirestoreQueryDirection::Ascending),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ]),
        )
        .await?;

    for doc in &docs {
        if let Ok(mut msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            let (_, docid) = doc.name.rsplit_once('/').unwrap();
            let (_, userid) = docid.rsplit_once('.').unwrap();
            let new_docid = format!("{dst_scenario_name}.{userid}");
            log::info!("copying {} to {}", docid, new_docid);
            msg.scenario_name = dst_scenario_name.into();
            db.update_obj("leaderboard", &new_docid, &msg, None).await?;
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    Ok(())
}
