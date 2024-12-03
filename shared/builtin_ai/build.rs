use anyhow::{Result, Context};
use glob::glob;
use libflate::gzip::{EncodeOptions, Encoder, HeaderBuilder};
use rayon::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use tar::Header;

thread_local! {
    static COMPILERS: std::cell::RefCell<oort_compiler::Compiler> = RefCell::new({
        let mut x = oort_compiler::Compiler::new();
        x.enable_online();
        x
    });
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default()).init();

    std::env::vars().for_each(|(k, v)| {
        log::info!("Environment: {}={}", k, v);
    });

    log::info!("current dir: {:?}", std::env::current_dir()?);
    let input = "src";
    let output = "../../target/builtin-ai.tar.gz";
    std::fs::create_dir_all("../../target")?;

    let directory_paths = glob(&format!("{}/**/*", input))?
        .map(|x| x.unwrap())
        .filter(|x| x.is_dir())
        .collect::<Vec<_>>();
    for directory_path in directory_paths {
        println!("cargo:rerun-if-changed={}", directory_path.display());
    }

    let paths: Vec<_> = glob(&format!("{}/**/*.rs", input))?
        .map(|x| x.unwrap())
        .filter(|x| !["lib.rs", "mod.rs"].contains(&x.file_name().unwrap().to_str().unwrap()))
        .collect();

    let results: Vec<_> = paths
        .par_iter()
        .map(
            |path| -> Result<(PathBuf, /*rust*/ String, /*wasm*/ Vec<u8>)> {
                let source_code = std::fs::read_to_string(path).unwrap();
                let wasm = COMPILERS
                    .with(|compiler_cell| compiler_cell.borrow_mut().compile(&source_code))?;
                let optimized_wasm = wasm_opt(&wasm).context("wasm-opt")?;
                println!("cargo::rerun-if-changed={}", path.display());
                log::info!(
                    "{} source {}K wasm {}K optimized {}K",
                    path.display(),
                    source_code.len() / 1000,
                    wasm.len() / 1000,
                    optimized_wasm.len() / 1000
                );
                Ok((path.clone(), source_code, optimized_wasm))
            },
        )
        .collect();

    let writer = std::fs::File::create(output)?;
    let header = HeaderBuilder::new().modification_time(0).finish();
    let options = EncodeOptions::new().header(header);
    let encoder = Encoder::with_options(writer, options).unwrap();
    let mut ar = tar::Builder::new(encoder);

    for r in results.iter() {
        let (path, source_code, _) = r.as_ref().unwrap();
        let path = path.strip_prefix(input)?;
        let data = source_code.as_bytes();
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o700);
        header.set_cksum();
        ar.append_data(&mut header, path, data).unwrap();
    }

    for r in results.iter() {
        let (path, _, wasm) = r.as_ref().unwrap();
        let mut path = path.strip_prefix(input)?.to_path_buf();
        path.set_extension("wasm");
        let mut header = Header::new_gnu();
        header.set_size(wasm.len() as u64);
        header.set_mode(0o700);
        header.set_cksum();
        ar.append_data(&mut header, path, &wasm[..]).unwrap();
    }

    let encoder = ar.into_inner()?;
    encoder.finish();

    Ok(())
}

fn wasm_opt(wasm: &[u8]) -> Result<Vec<u8>> {
    let tmp_dir = tempdir::TempDir::new("oort_wasm_opt").unwrap();
    let infile = tmp_dir.path().join("input.wasm");
    let outfile = tmp_dir.path().join("output.wasm");
    std::fs::write(&infile, wasm).context("writing input file")?;
    let output = std::process::Command::new("wasm-opt")
        .args(["-Oz", "-o", outfile.as_os_str().to_str().unwrap(), infile.as_os_str().to_str().unwrap()])
        .output().context("spawning wasm-opt")?;
    if !output.status.success() {
        anyhow::bail!("wasm-opt failed with stderr: {}", std::str::from_utf8(&output.stderr)?);
    }
    let output = std::fs::read(outfile).context("reading output file")?;
    Ok(output)
}
