use anyhow::Result;
use clap::Parser as _;
use rayon::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("check_compiler_determinism=info"),
    )
    .init();

    #[derive(clap::Parser, Debug)]
    struct Arguments {
        srcs: Vec<String>,
        #[clap(short, long, default_value = "/tmp/oort-compiler-determinism")]
        out_dir: String,
    }
    let args = Arguments::parse();

    let results: Vec<_> = args
        .srcs
        .par_iter()
        .map(|src| -> Result<()> {
            let src_code = std::fs::read_to_string(src).unwrap();
            let mut compiler0 = oort_compiler::Compiler::new();
            let mut compiler1 = oort_compiler::Compiler::new();
            let wasms = [
                compiler0.compile(&src_code)?,
                compiler1.compile(&src_code)?,
                compiler1.compile(&src_code)?,
            ];
            let mut diffs: Vec<usize> = wasms
                .iter()
                .map(|x| *x == wasms[0])
                .enumerate()
                .filter(|x| !x.1)
                .map(|x| x.0)
                .collect();
            if diffs.is_empty() {
                println!("{src} identical");
                Ok(())
            } else {
                diffs.insert(0, 0);

                let mut out_path = std::path::PathBuf::new();
                out_path.push(&args.out_dir);
                std::fs::create_dir_all(&out_path)?;

                let mut src_path = PathBuf::new();
                src_path.push(src);
                src_path.set_extension("wat");
                out_path.push(src_path.file_name().unwrap());

                println!(
                    "{src} differs, wrote to {}.{{{}}}",
                    out_path.display(),
                    diffs
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );

                out_path.set_extension("wat.0");
                for d in diffs.iter() {
                    out_path.set_extension(d.to_string());
                    std::fs::write(&out_path, wasm2wat(&wasms[*d]))?;
                }

                Err(anyhow::anyhow!("Compiler was not deterministic"))
            }
        })
        .collect();
    for result in results {
        result?;
    }
    Ok(())
}

fn wasm2wat(wasm: &[u8]) -> Vec<u8> {
    wabt::Wasm2Wat::new()
        .convert(wasm)
        .unwrap()
        .as_ref()
        .to_vec()
}
