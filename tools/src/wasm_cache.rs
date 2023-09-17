use std::{fs, path::PathBuf};

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
