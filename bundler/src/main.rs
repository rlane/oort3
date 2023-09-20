use anyhow::Result;
use clap::Parser as _;
use notify::{RecursiveMode, Watcher};
use std::time::Duration;
use std::{collections::HashMap, path::Path};

#[derive(clap::Parser, Debug)]
struct Arguments {
    files: Vec<String>,

    #[clap(short, long)]
    main: Option<String>,

    #[clap(short, long)]
    output: String,

    #[clap(short, long)]
    watch: bool,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("bundler=info"))
        .init();

    let args = Arguments::parse();

    let main = args.main.unwrap_or_else(|| "".to_owned());

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(_) => tx.send(()).unwrap(),
        Err(e) => println!("watch error: {:?}", e),
    })?;
    for f in &args.files {
        watcher.watch(Path::new(f), RecursiveMode::NonRecursive)?;
    }

    loop {
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

        std::fs::write(&args.output, joined.finalize(&main))?;

        if !args.watch {
            break;
        }

        rx.recv().unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
