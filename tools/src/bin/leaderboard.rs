use chrono::prelude::*;
use clap::{Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::LeaderboardSubmission;

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
    List {
        scenario: String,
        #[clap(short = 'n', long, value_parser, default_value_t = 10)]
        limit: usize,
    },
    Download {
        #[clap(short, long)]
        scenario: Option<String>,
        #[clap(short, long)]
        user: Option<String>,
        #[clap(short = 'n', long, value_parser, default_value_t = 10)]
        limit: usize,
        #[clap(short, long, value_parser)]
        out_dir: String,
    },
    Get {
        docid: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let args = Arguments::parse();
    match args.cmd {
        SubCommand::List { scenario, limit } => cmd_list(&args.project_id, &scenario, limit).await,
        SubCommand::Download {
            user,
            scenario,
            limit,
            out_dir,
        } => cmd_download(&args.project_id, &user, &scenario, limit, &out_dir).await,
        SubCommand::Get { docid } => cmd_get(&args.project_id, docid).await,
    }
}

async fn cmd_list(
    project_id: &str,
    scenario_name: &str,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;

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
                    FirestoreQueryOrder::new("time".to_owned(), FirestoreQueryDirection::Ascending),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ])
                .with_limit(limit as u32),
        )
        .await?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Rank", "User", "Time", "Docid", "Created"]);

    for (i, doc) in docs.iter().enumerate() {
        let (_, docid) = doc.name.rsplit_once('/').unwrap();
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            let datetime: DateTime<Local> = DateTime::from(msg.timestamp);
            table.add_row(vec![
                format!("{}", i + 1),
                msg.username.to_owned(),
                format!("{:.3}s", msg.time),
                docid.to_owned(),
                datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
            ]);
        }
    }

    println!("Scenario: {scenario_name}");
    println!("{table}");

    Ok(())
}

async fn cmd_download(
    project_id: &str,
    username: &Option<String>,
    scenario_name: &Option<String>,
    limit: usize,
    out_dir: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut filters = vec![];
    if let Some(username) = username {
        filters.push(FirestoreQueryFilter::Compare(Some(
            FirestoreQueryFilterCompare::Equal("username".into(), username.into()),
        )));
    }
    if let Some(scenario_name) = scenario_name {
        filters.push(FirestoreQueryFilter::Compare(Some(
            FirestoreQueryFilterCompare::Equal("scenario_name".into(), scenario_name.into()),
        )));
    }

    let db = FirestoreDb::new(project_id).await?;

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new("leaderboard".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(
                        filters,
                        FirestoreQueryFilterCompositeOperator::And,
                    ),
                ))
                .with_order_by(vec![
                    FirestoreQueryOrder::new("time".to_owned(), FirestoreQueryDirection::Ascending),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ])
                .with_limit(limit as u32),
        )
        .await?;

    std::fs::create_dir_all(out_dir).unwrap();
    for doc in docs.iter() {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            let filename = format!("{}/{}.{}.rs", &out_dir, msg.scenario_name, msg.username);
            std::fs::write(&filename, &msg.code).unwrap();
            println!("Wrote {filename}");
        }
    }

    Ok(())
}

async fn cmd_get(
    project_id: &str,
    docid: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;
    if let Ok(msg) = db
        .get_obj::<LeaderboardSubmission, _>("leaderboard", &docid)
        .await
    {
        let datetime: DateTime<Local> = DateTime::from(msg.timestamp);
        println!("// User: {}", msg.username);
        println!("// Scenario: {}", msg.scenario_name);
        println!("// Date: {datetime}");
        println!("// Time: {:.3}s Size: {}", msg.time, msg.code_size);
        println!("{}", msg.code.trim());
    } else {
        let doc = db.get_doc("leaderboard", &docid, None).await?;
        println!("Failed to parse {doc:?}");
    }

    Ok(())
}
