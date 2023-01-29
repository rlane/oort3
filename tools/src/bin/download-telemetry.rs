use chrono::prelude::*;
use clap::Parser;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{Telemetry, TelemetryMsg};

const COLLECTION_NAME: &str = "telemetry";

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,
    #[clap(long, value_parser, default_value_t = String::from("scratch/telemetry.sqlite"))]
    db: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("download_telemetry=info"),
    )
    .init();

    let args = Arguments::parse();
    let db = FirestoreDb::new(&args.project_id).await?;
    let mut sqlite = rusqlite::Connection::open(&args.db)?;
    let transaction = sqlite.transaction()?;

    transaction.execute(
        "CREATE TABLE IF NOT EXISTS SyncTime (
            id INTEGER PRIMARY KEY,
            timestamp TEXT
        )",
        (),
    )?;

    transaction.execute(
        "CREATE TABLE IF NOT EXISTS StartScenario (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            userid TEXT,
            username TEXT,
            build TEXT,
            scenario_name TEXT,
            code TEXT
        )",
        (),
    )?;

    transaction.execute(
        "CREATE TABLE IF NOT EXISTS FinishScenario (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            userid TEXT,
            username TEXT,
            build TEXT,
            scenario_name TEXT,
            code TEXT,
            success INTEGER,
            time REAL
        )",
        (),
    )?;

    transaction.execute(
        "CREATE TABLE IF NOT EXISTS Crash (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            userid TEXT,
            username TEXT,
            build TEXT,
            msg TEXT
        )",
        (),
    )?;

    transaction.execute(
        "CREATE TABLE IF NOT EXISTS SubmitToTournament (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            userid TEXT,
            username TEXT,
            build TEXT,
            scenario_name TEXT,
            code TEXT
        )",
        (),
    )?;

    transaction.execute(
        "CREATE TABLE IF NOT EXISTS Feedback (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            userid TEXT,
            username TEXT,
            build TEXT,
            text TEXT
        )",
        (),
    )?;

    let last_sync_timestamp = {
        let mut last_sync_timestamp_text = "1970-01-01T00:00:00Z".to_string();
        let mut stmt = transaction
            .prepare("SELECT timestamp FROM SyncTime ORDER BY datetime(timestamp) DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            last_sync_timestamp_text = row.get(0)?;
        }
        let last_sync_timestamp: DateTime<Utc> =
            DateTime::parse_from_rfc3339(&last_sync_timestamp_text)?.into();
        log::info!(
            "Querying for telemetry since {}",
            last_sync_timestamp.to_rfc3339()
        );
        last_sync_timestamp
    };

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(COLLECTION_NAME.into())
                .with_filter(FirestoreQueryFilter::Compare(Some(
                    FirestoreQueryFilterCompare::GreaterThan(
                        "timestamp".into(),
                        last_sync_timestamp.timestamp_millis().into(),
                    ),
                )))
                .with_order_by(vec![FirestoreQueryOrder::new(
                    "timestamp".to_owned(),
                    FirestoreQueryDirection::Descending,
                )])
                .clone(),
        )
        .await?;
    log::info!("Found {} docs", docs.len());

    let mut new_sync_timestamp = last_sync_timestamp;

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            if new_sync_timestamp < msg.timestamp {
                new_sync_timestamp = msg.timestamp;
            }
            match &msg.payload {
                Telemetry::StartScenario {
                    scenario_name,
                    code,
                    ..
                } => {
                    transaction.execute(
                            "INSERT INTO StartScenario (timestamp, userid, username, build, scenario_name, code) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            (&msg.timestamp.to_rfc3339(), &msg.userid, &msg.username, &msg.build, scenario_name, code))?;
                }
                Telemetry::FinishScenario {
                    scenario_name,
                    code,
                    success,
                    time,
                    ..
                } => {
                    transaction.execute(
                            "INSERT INTO FinishScenario (timestamp, userid, username, build, scenario_name, code, success, time) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            (&msg.timestamp.to_rfc3339(), &msg.userid, &msg.username, &msg.build, scenario_name, code, success, time))?;
                }
                Telemetry::Crash { msg: crash_msg } => {
                    transaction.execute(
                            "INSERT INTO Crash (timestamp, userid, username, build, msg) VALUES (?1, ?2, ?3, ?4, ?5)",
                            (&msg.timestamp.to_rfc3339(), &msg.userid, &msg.username, &msg.build, crash_msg))?;
                }
                Telemetry::SubmitToTournament {
                    scenario_name,
                    code,
                } => {
                    transaction.execute(
                            "INSERT INTO SubmitToTournament (timestamp, userid, username, build, scenario_name, code) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            (&msg.timestamp.to_rfc3339(), &msg.userid, &msg.username, &msg.build, scenario_name, code))?;
                }
                Telemetry::Feedback { text } => {
                    transaction.execute(
                            "INSERT INTO Feedback (timestamp, userid, username, build, text) VALUES (?1, ?2, ?3, ?4, ?5)",
                            (&msg.timestamp.to_rfc3339(), &msg.userid, &msg.username, &msg.build, text))?;
                }
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    log::info!("New sync timestamp: {}", new_sync_timestamp.to_rfc3339());
    transaction.execute(
        "INSERT INTO SyncTime (timestamp) VALUES (?1)",
        (&new_sync_timestamp.to_rfc3339(),),
    )?;

    transaction.commit()?;

    Ok(())
}
