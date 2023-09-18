use sha2::{Digest, Sha256};
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

    fn key(source_code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source_code.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    pub fn get(&self, name: &str, source_code: &str) -> Option<Vec<u8>> {
        let key = Self::key(source_code);
        let path = self.path.join(format!("{key}.wasm"));
        let binary_path = std::env::current_exe().unwrap();

        let cache_file_ts = fs::metadata(&path).ok().and_then(|x| x.modified().ok());
        let binary_ts = fs::metadata(binary_path)
            .ok()
            .and_then(|x| x.modified().ok())
            .unwrap();

        if cache_file_ts.is_none() {
            log::info!("WASM cache miss for {:?} ({})", name, key);
            return None;
        }

        if binary_ts >= cache_file_ts.unwrap() {
            log::info!("WASM cache is stale (binary) for {:?}", name);
            return None;
        }

        log::info!("WASM cache hit for {:?}", name);
        fs::read(&path).ok()
    }

    pub fn put(&self, source_code: &str, bytes: &[u8]) {
        let key = Self::key(source_code);
        let path = self.path.join(format!("{key}.wasm"));
        if let Err(e) = fs::write(path, bytes) {
            log::warn!("Failed to write to WASM cache: {:?}", e);
        }
        if let Err(e) = self.expire() {
            log::warn!("Failed to expire WASM cache: {:?}", e);
        }
    }

    fn expire(&self) -> anyhow::Result<()> {
        let mut entries = fs::read_dir(&self.path)?.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|x| x.metadata().unwrap().modified().unwrap());
        entries.reverse();
        for entry in entries.into_iter().skip(10) {
            fs::remove_file(entry.path())?;
        }
        Ok(())
    }
}
