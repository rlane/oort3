use chrono::prelude::*;
use clap::{Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_telemetry_proto::{Telemetry, TelemetryMsg};
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
        SubCommand::List { user } => cmd_list(&args.project_id, user).await,
        SubCommand::Get { docid } => cmd_get(&args.project_id, docid).await,
        SubCommand::Top { scenario, out_dir } => cmd_top(&args.project_id, scenario, out_dir).await,
    }
}

async fn cmd_list(
    project_id: &str,
    user_filter: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;

    let mut docs: Vec<Document> = db
        .query_doc(FirestoreQueryParams::new(COLLECTION_NAME.into()).clone())
        .await?;
    docs.sort_by_key(|doc| doc.create_time.clone().map(|x| x.seconds).unwrap_or(0));
    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let (_, docid) = doc.name.rsplit_once('/').unwrap();
            let user = msg.username.as_ref().unwrap_or(&msg.userid);
            if let Some(u) = user_filter.as_ref() {
                if user != u {
                    continue;
                }
            }
            let epoch_time = doc.create_time.clone().map(|x| x.seconds).unwrap_or(0);
            let time = Local.timestamp(epoch_time, 0);
            let prefix = format!("{docid} {}", time.format("%Y-%m-%d %H:%M:%S"));
            match &msg.payload {
                Telemetry::StartScenario { scenario_name, .. } => {
                    println!("{prefix} StartScenario user={user} scenario={scenario_name}")
                }
                Telemetry::FinishScenario {
                    scenario_name,
                    success,
                    ticks,
                    ..
                } => {
                    let ticks = if success.unwrap_or(false) {
                        ticks.to_string()
                    } else {
                        "failed".to_string()
                    };
                    println!(
                        "{prefix} FinishScenario user={user} scenario={scenario_name} ticks={ticks}"
                    );
                }
                Telemetry::Crash { .. } => println!("{prefix} Crash user={user}"),
                Telemetry::SubmitToTournament { scenario_name, .. } => {
                    println!("{prefix} SubmitToTournament user={user} scenario={scenario_name}")
                }
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
        let user = msg.username.as_ref().unwrap_or(&msg.userid);
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
                ticks,
                code_size,
                success,
            } => {
                println!("// User: {}", user);
                println!("// Scenario: {}", scenario_name);
                println!(
                    "// Success: {} Ticks: {} Size: {}",
                    success.unwrap_or(false),
                    ticks,
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

    // userid -> (ticks, creation_time, docid, msg)
    let mut best_times: HashMap<String, (u32, DateTime<Local>, String, TelemetryMsg)> =
        HashMap::new();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let (_, docid) = doc.name.rsplit_once('/').unwrap();
            let epoch_time = doc.create_time.clone().map(|x| x.seconds).unwrap_or(0);
            let creation_time = Local.timestamp(epoch_time, 0);
            match &msg.payload {
                Telemetry::FinishScenario { ticks, .. } => {
                    let insert = if let Some((ref old_ticks, _, _, _)) = best_times.get(&msg.userid)
                    {
                        old_ticks > ticks
                    } else {
                        true
                    };
                    if insert {
                        best_times.insert(
                            msg.userid.clone(),
                            (*ticks, creation_time, docid.to_owned(), msg.clone()),
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
    top.sort_by_key(|x| x.0);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Rank", "User", "Time", "Docid", "Created"]);

    let mut outputs: Vec<(String, String)> = Vec::new();

    for (i, (_, creation_time, docid, msg)) in top.iter().take(10).enumerate() {
        let user = msg.username.as_ref().unwrap_or(&msg.userid);
        match &msg.payload {
            Telemetry::FinishScenario { ticks, code, .. } => {
                table.add_row(vec![
                    format!("{}", i + 1),
                    user.to_owned(),
                    format!("{:.2}s", *ticks as f64 / 60.0),
                    docid.to_owned(),
                    creation_time.format("%Y-%m-%d %H:%M:%S").to_string(),
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
