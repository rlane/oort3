use anyhow::Result;
use clap::Parser as _;
use std::collections::HashMap;

#[derive(clap::Parser, Debug)]
struct Arguments {
    files: Vec<String>,

    #[clap(short, long)]
    output: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("bundler=info"))
        .init();

    let args = Arguments::parse();

    let files = args
        .files
        .iter()
        .map(|f| {
            (
                std::path::Path::new(f)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                std::fs::read_to_string(f).unwrap(),
            )
        })
        .collect::<HashMap<_, _>>();

    let joined = oort_multifile::join(files)?;

    std::fs::write(&args.output, joined)?;

    Ok(())
}
