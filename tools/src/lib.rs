use std::{fs, path::Path, path::PathBuf};

use oort_simulator::simulation::Code;

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
        ("https://compiler.oort.rs", "https://shortcode.oort.rs")
    };

    let wasm_cache = wasm_cache.and_then(|path| WasmCache::new(path.to_owned()));
    if let Some(wasm_cache) = wasm_cache.as_ref() {
        if let Some(wasm) = wasm_cache.get(shortcode) {
            return Ok(AI {
                name,
                source_code: format!("// read from cache: {:?}", wasm_cache.path),
                compiled_code: Code::Wasm(wasm),
            });
        }
    }

    let source_code = if std::fs::metadata(shortcode).ok().is_some() {
        std::fs::read_to_string(shortcode).unwrap()
    } else {
        log::info!("Fetching {:?}", shortcode);
        http.get(&format!("{shortcode_url}/shortcode/{shortcode}"))
            .send()
            .await?
            .text()
            .await?
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
        wasm_cache.put(shortcode, &compiled_code);
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

pub struct WasmCache {
    path: PathBuf,
}

impl WasmCache {
    pub fn new(path: PathBuf) -> Option<Self> {
        if let Err(e) = fs::create_dir_all(&path) {
            log::warn!("Failed to create WASM cache directory: {:?}", e);
            return None;
        }
        Some(Self { path: path.clone() })
    }

    fn key(shortcode: &str) -> String {
        shortcode.replace('/', "_")
    }

    pub fn get(&self, shortcode: &str) -> Option<Vec<u8>> {
        let key = Self::key(shortcode);
        let path = self.path.join(format!("{key}.wasm"));
        let binary_path = std::env::current_exe().unwrap();

        let local_file_ts = fs::metadata(shortcode).ok().and_then(|x| x.modified().ok());
        let cache_file_ts = fs::metadata(&path).ok().and_then(|x| x.modified().ok());
        let binary_ts = fs::metadata(binary_path)
            .ok()
            .and_then(|x| x.modified().ok())
            .unwrap();

        if cache_file_ts.is_none() {
            log::info!("WASM cache miss for {:?}", shortcode);
            return None;
        }

        if binary_ts >= cache_file_ts.unwrap() {
            log::info!("WASM cache is stale (binary) for {:?}", shortcode);
            return None;
        }

        if let Some(local_ts) = local_file_ts {
            if local_ts >= cache_file_ts.unwrap() {
                log::info!("WASM cache is stale (source) for {:?}", shortcode);
                return None;
            }
        }

        log::info!("WASM cache hit for {:?}", shortcode);
        fs::read(&path).ok()
    }

    pub fn put(&self, shortcode: &str, bytes: &[u8]) {
        let key = Self::key(shortcode);
        let path = self.path.join(format!("{key}.wasm"));
        if let Err(e) = fs::write(path, bytes) {
            log::warn!("Failed to write to WASM cache: {:?}", e);
        }
    }
}
