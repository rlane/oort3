use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("prepare_compiler_directory=info"),
    )
    .init();

    #[derive(clap::Parser, Debug)]
    struct Arguments {
        #[clap(short, long, default_value = "/tmp/oort-ai")]
        dir: String,
    }
    let args = Arguments::parse();

    std::fs::remove_dir_all(&args.dir)?;
    std::fs::create_dir_all(&args.dir)?;
    let mut compiler = oort_compiler::Compiler::new_with_dir(std::path::Path::new(&args.dir));
    compiler.compile(include_str!("../../../shared/builtin_ai/src/empty.rs"))?;
    Ok(())
}
