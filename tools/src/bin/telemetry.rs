use chrono::prelude::*;
use clap::{Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{Telemetry, TelemetryMsg};
use std::collections::HashMap;

const COLLECTION_NAME: &str = "telemetry";

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
        #[clap(short, long, value_parser)]
        user: Option<String>,
        #[clap(short = 'n', long, value_parser, default_value_t = 100)]
        limit: usize,
    },
    Get {
        docid: String,
    },
    Top {
        scenario: String,
        #[clap(short, long, value_parser)]
        out_dir: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let args = Arguments::parse();
    match args.cmd {
        SubCommand::List { user, limit } => cmd_list(&args.project_id, user, limit).await,
        SubCommand::Get { docid } => cmd_get(&args.project_id, docid).await,
        SubCommand::Top { scenario, out_dir } => cmd_top(&args.project_id, scenario, out_dir).await,
    }
}

async fn cmd_list(
    project_id: &str,
    user_filter: Option<String>,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;

    let mut docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(COLLECTION_NAME.into())
                .with_order_by(vec![FirestoreQueryOrder::new(
                    "timestamp".to_owned(),
                    FirestoreQueryDirection::Descending,
                )])
                .with_limit(limit as u32)
                .clone(),
        )
        .await?;
    docs.sort_by_key(|doc| doc.create_time.clone().map(|x| x.seconds).unwrap_or(0));
    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let (_, docid) = doc.name.rsplit_once('/').unwrap();
            let user = &msg.username;
            if let Some(u) = user_filter.as_ref() {
                if user != u {
                    continue;
                }
            }
            let datetime: DateTime<Local> = DateTime::from(msg.timestamp);
            let prefix = format!("{docid} {}", datetime.format("%Y-%m-%d %H:%M:%S"));
            match &msg.payload {
                Telemetry::StartScenario { scenario_name, .. } => {
                    println!("{prefix} StartScenario user={user} scenario={scenario_name}")
                }
                Telemetry::FinishScenario {
                    scenario_name,
                    success,
                    time,
                    ..
                } => {
                    let time = if *success {
                        format!("{:.2}s", time.unwrap_or_default())
                    } else {
                        "failed".to_string()
                    };
                    println!(
                        "{prefix} FinishScenario user={user} scenario={scenario_name} time={time}"
                    );
                }
                Telemetry::Crash { .. } => println!("{prefix} Crash user={user}"),
                Telemetry::SubmitToTournament { scenario_name, .. } => {
                    println!("{prefix} SubmitToTournament user={user} scenario={scenario_name}")
                }
                Telemetry::Feedback { .. } => println!("{prefix} Feedback user={user}"),
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    Ok(())
}

async fn cmd_get(
    project_id: &str,
    docid: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;
    if let Ok(msg) = db.get_obj::<TelemetryMsg>(COLLECTION_NAME, &docid).await {
        let user = &msg.username;
        match msg.payload {
            Telemetry::StartScenario {
                scenario_name,
                code,
            } => {
                println!("// User: {}", user);
                println!("// Scenario: {}", scenario_name);
                println!("{}", code.trim());
            }
            Telemetry::FinishScenario {
                scenario_name,
                code,
                time,
                code_size,
                success,
                ..
            } => {
                println!("// User: {}", user);
                println!("// Scenario: {}", scenario_name);
                println!(
                    "// Success: {} Time: {:.2}s Size: {}",
                    success,
                    time.unwrap_or_default(),
                    code_size
                );
                println!("{}", code.trim());
            }
            Telemetry::Crash { msg } => println!("Crash: {msg}"),
            Telemetry::SubmitToTournament {
                scenario_name,
                code,
            } => {
                println!("// User: {}", user);
                println!("// Scenario: {}", scenario_name);
                println!("{}", code.trim());
            },
            Telemetry::Feedback {
                text,
            } => {
                let datetime: DateTime<Local> = DateTime::from(msg.timestamp);
                println!("// User: {}", user);
                println!("// Date: {}", datetime);
                println!("// Build: {}", msg.build);
                println!("{}", text.trim());
            }
        }
    } else {
        let doc = db.get_doc_by_id("", COLLECTION_NAME, &docid).await?;
        println!("Failed to parse {:?}", doc);
    }

    Ok(())
}

async fn cmd_top(
    project_id: &str,
    scenario: String,
    out_dir: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(COLLECTION_NAME.into()).with_filter(
                FirestoreQueryFilter::Composite(FirestoreQueryFilterComposite::new(vec![
                    FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                        "type".into(),
                        "FinishScenario".into(),
                    ))),
                    FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                        "success".into(),
                        true.into(),
                    ))),
                    FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                        "scenario_name".into(),
                        scenario.clone().into(),
                    ))),
                ])),
            ),
        )
        .await?;

    // userid -> (time, timestamp, docid, msg)
    let mut best_times: HashMap<String, (f64, DateTime<Utc>, String, TelemetryMsg)> =
        HashMap::new();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let (_, docid) = doc.name.rsplit_once('/').unwrap();
            match &msg.payload {
                Telemetry::FinishScenario { time, .. } => {
                    let insert = if let Some((ref old_time, _, _, _)) = best_times.get(&msg.userid)
                    {
                        *old_time > time.unwrap_or_default()
                    } else {
                        true
                    };
                    if insert {
                        best_times.insert(
                            msg.userid.clone(),
                            (
                                time.unwrap_or_default(),
                                msg.timestamp,
                                docid.to_owned(),
                                msg.clone(),
                            ),
                        );
                    }
                }
                _ => unreachable!(),
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    let mut top: Vec<_> = best_times.into_values().collect();
    top.sort_by_key(|x| (x.0 * 1000.0) as i64);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Rank", "User", "Time", "Docid", "Created"]);

    let mut outputs: Vec<(String, String)> = Vec::new();

    for (i, (_, timestamp, docid, msg)) in top.iter().take(10).enumerate() {
        let user = &msg.username;
        let datetime: DateTime<Local> = DateTime::from(*timestamp);
        match &msg.payload {
            Telemetry::FinishScenario { time, code, .. } => {
                table.add_row(vec![
                    format!("{}", i + 1),
                    user.to_owned(),
                    format!("{:.2}s", time.unwrap_or_default()),
                    docid.to_owned(),
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                ]);
                outputs.push((user.to_owned(), code.to_owned()));
            }
            _ => unreachable!(),
        }
    }

    println!("Scenario: {}", scenario);
    println!("{}", table);

    if let Some(out_dir) = out_dir {
        std::fs::create_dir_all(&out_dir).unwrap();
        for (user, code) in outputs.iter() {
            std::fs::write(format!("{}/{}.rs", &out_dir, user), code).unwrap();
        }
    }

    Ok(())
}
