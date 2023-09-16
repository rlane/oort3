use std::{fs, path::Path};

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
    let (compiler_url, shortcode_url) = if dev {
        ("http://localhost:8081", "http://localhost:8084")
    } else {
        ("https://compiler.oort.rs", "https://shortcode.oort.rs")
    };
    let wasm_cache = wasm_cache.map(|path| path.join(format!("{shortcode}.wasm")));
    let local_file_meta = fs::metadata(shortcode).ok();
    let cache_file_meta = wasm_cache.as_ref().and_then(|p| fs::metadata(p).ok());

    let cache_is_fresh = if let Some(cache) = &cache_file_meta {
        if let Some(local) = &local_file_meta {
            cache.modified()? > local.modified()?
        } else {
            true
        }
    } else {
        false
    };

    let (compiled_code, source_code) = if cache_is_fresh {
        let cache = wasm_cache.as_ref().unwrap();
        log::info!("Reading cache file {:?}", cache);
        (fs::read(cache)?, format!("// read from cache: {:?}", cache))
    } else {
        let source_code = if local_file_meta.is_some() {
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

        (response.bytes().await?.to_vec(), source_code)
    };

    if let Some(cache_file) = wasm_cache {
        if !cache_is_fresh {
            fs::write(cache_file, &compiled_code)?;
        }
    }

    let compiled_code = oort_simulator::vm::precompile(&compiled_code).unwrap();

    Ok(AI {
        name: shortcode.to_string(),
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
