use chrono::prelude::*;
use clap::{Parser, Subcommand};
use firestore::*;
use oort_proto::ShortcodeUpload;

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
    Get { docid: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("shortcode=info"))
        .init();

    let args = Arguments::parse();
    match args.cmd {
        SubCommand::Get { docid } => cmd_get(&args.project_id, docid).await,
    }
}
async fn cmd_get(
    project_id: &str,
    docid: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;
    if let Ok(msg) = db.get_obj::<ShortcodeUpload, _>("shortcode", &docid).await {
        let datetime: DateTime<Local> = DateTime::from(msg.timestamp);
        println!("// User: {}", msg.username);
        println!("// Date: {datetime}");
        println!("{}", msg.code.trim());
    } else {
        let doc = db.get_doc("shortcode", &docid, None).await?;
        println!("Failed to parse {doc:?}");
    }

    Ok(())
}
