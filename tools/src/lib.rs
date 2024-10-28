mod wasm_cache;

use oort_compiler::Compiler;
use oort_simulator::simulation::Code;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct AI {
    pub name: String,
    pub source_code: String,
    pub compiled_code: Code,
}

pub async fn fetch_and_compile(
    http: &reqwest::Client,
    shortcode: &str,
    dev: bool,
    wasm_cache: Option<&Path>,
) -> anyhow::Result<AI> {
    let name = shortcode.rsplit('/').next().unwrap().to_string();
    let (compiler_url, shortcode_url) = if dev {
        ("http://localhost:8081", "http://localhost:8084")
    } else {
        ("https://compiler.oort.rs", "https://backend.oort.rs")
    };

    let wasm_cache = wasm_cache.and_then(|path| wasm_cache::WasmCache::new(path.to_owned()));
    let get_cached = |cache_key: &str| {
        if let Some(wasm_cache) = wasm_cache.as_ref() {
            if let Some(wasm) = wasm_cache.get(shortcode, cache_key) {
                return Some(AI {
                    name: name.clone(),
                    source_code: "".to_string(),
                    compiled_code: Code::Wasm(wasm),
                });
            }
        }
        None
    };

    let (cache_key, source_code) = if std::fs::metadata(shortcode).ok().is_some() {
        let source_code = read_filesystem(shortcode)?;
        let cache_key = source_code.clone();
        if let Some(ai) = get_cached(&cache_key) {
            return Ok(ai);
        }
        (cache_key, source_code)
    } else {
        log::info!("Fetching {:?}", shortcode);
        let cache_key = shortcode.to_string();
        if let Some(ai) = get_cached(&cache_key) {
            return Ok(ai);
        }
        let source_code = http
            .get(&format!("{shortcode_url}/shortcode/{shortcode}"))
            .send()
            .await?
            .text()
            .await?;
        (cache_key, source_code)
    };

    log::info!("Compiling {:?}", shortcode);

    let response = http
        .post(&format!("{compiler_url}/compile"))
        .body(source_code.clone())
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to compile {:?}: {:?}",
            shortcode,
            response.text().await?
        );
    }

    let compiled_code = response.bytes().await?.to_vec();

    if let Some(wasm_cache) = wasm_cache {
        wasm_cache.put(&cache_key, &compiled_code);
    }

    let compiled_code = oort_simulator::vm::precompile(&compiled_code).unwrap();

    Ok(AI {
        name,
        source_code,
        compiled_code,
    })
}

pub async fn fetch_and_compile_multiple(
    http: &reqwest::Client,
    shortcodes: &[String],
    dev: bool,
    wasm_cache: Option<&Path>,
) -> anyhow::Result<Vec<AI>> {
    let futures = shortcodes
        .iter()
        .map(|shortcode| fetch_and_compile(http, shortcode, dev, wasm_cache));
    let results = futures::future::join_all(futures).await;
    results.into_iter().collect()
}

pub struct ParallelCompiler {
    sender: Mutex<std::sync::mpsc::Sender<Compiler>>,
    receiver: Mutex<std::sync::mpsc::Receiver<Compiler>>,
}

impl ParallelCompiler {
    pub fn new(parallelism: usize) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        for _ in 0..parallelism {
            let compiler = Compiler::new();
            sender.send(compiler).unwrap();
        }
        Self {
            sender: Mutex::new(sender),
            receiver: Mutex::new(receiver),
        }
    }

    pub fn compile(&self, source_code: &str) -> anyhow::Result<Vec<u8>> {
        let mut compiler = self.receiver.lock().unwrap().recv().unwrap();
        let ret = compiler.compile(source_code)?;
        self.sender.lock().unwrap().send(compiler)?;
        Ok(ret)
    }
}

pub fn read_filesystem(path: &str) -> anyhow::Result<String> {
    let mut pathbuf = PathBuf::from(path);
    let mut metadata = std::fs::metadata(&pathbuf)
        .map_err(|e| anyhow::anyhow!("Failed to read {:?}: {:?}", pathbuf, e.to_string()))?;
    if metadata.is_dir() {
        if let Ok(src_metadata) = std::fs::metadata(pathbuf.join("src")) {
            pathbuf.push("src");
            metadata = src_metadata;
        }
    }

    if metadata.is_file() {
        Ok(std::fs::read_to_string(&pathbuf)?)
    } else if metadata.is_dir() {
        let mut files = HashMap::new();
        for entry in std::fs::read_dir(&pathbuf)? {
            let entry = entry?;
            let path = entry.path();
            let extension = path.extension().unwrap_or_default();
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            if path.is_file()
                && extension == "rs"
                && !stem.starts_with('.')
                && !stem.ends_with("test")
            {
                log::info!("Reading {:?}", path);
                files.insert(
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    std::fs::read_to_string(path)?,
                );
            }
        }
        let multifile = oort_multifile::join(files)?;
        let main_filename = multifile.filenames.first().unwrap();
        multifile.finalize(main_filename)
    } else {
        anyhow::bail!("Not a file or directory: {:?}", path);
    }
}
