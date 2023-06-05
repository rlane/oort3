use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use idb::{Database, Factory, KeyPath, ObjectStoreParams, Query, Transaction, TransactionMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use wasm_bindgen::prelude::*;

const SCHEMA_VERSION: u32 = 3;
const VERSIONS: &str = "versions";
const CODE: &str = "code";

pub struct VersionControl {
    pub database: Database,
}

impl VersionControl {
    pub async fn new() -> Result<VersionControl, Error> {
        let factory = Factory::new()?;
        let mut open_request = factory.open("oort_version_control", Some(SCHEMA_VERSION))?;

        open_request.on_upgrade_needed(|event| {
            let database = event.database().unwrap();
            let transaction = event.transaction().unwrap().unwrap();
            if let Err(e) = Self::upgrade_database(&database, transaction) {
                log::error!("Error upgrading database: {:?}", e);
            }
        });

        let database = open_request.await?;
        Ok(VersionControl { database })
    }

    fn upgrade_database(database: &Database, transaction: Transaction) -> Result<(), Error> {
        if !database.store_names().contains(&VERSIONS.to_string()) {
            let mut store_params = ObjectStoreParams::new();
            store_params.key_path(Some(KeyPath::new_single("id")));
            database.create_object_store(VERSIONS, store_params)?;
        }

        {
            let store = transaction.object_store(VERSIONS)?;

            if !store.index_names().contains(&"scenario_name".to_string()) {
                store.create_index("scenario_name", KeyPath::new_single("scenario_name"), None)?;
            }

            if !store.index_names().contains(&"digest".to_string()) {
                store.create_index("digest", KeyPath::new_single("digest"), None)?;
            }
        }

        if !database.store_names().contains(&CODE.to_string()) {
            let store_params = ObjectStoreParams::new();
            database.create_object_store(CODE, store_params)?;
        }

        Ok(())
    }

    pub async fn create_version(&self, params: &CreateVersionParams) -> Result<(), Error> {
        let timestamp = chrono::Utc::now();
        let timestamp_string = timestamp.format("%Y%m%d-%H%M%S");
        let digest = digest(&params.code);
        let id = format!("{}-{}", timestamp_string, digest);
        let version = Version {
            id,
            scenario_name: params.scenario_name.clone(),
            timestamp,
            digest: digest.clone(),
            label: params.label.clone(),
        };
        let transaction = self
            .database
            .transaction(&[VERSIONS, CODE], TransactionMode::ReadWrite)?;
        let versions_store = transaction.object_store(VERSIONS)?;
        versions_store
            .add(&serde_wasm_bindgen::to_value(&version)?, None)
            .await?;
        let code_store = transaction.object_store(CODE)?;
        let compressed = {
            let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
            e.write_all(params.code.as_bytes())?;
            e.finish()?
        };
        let value: js_sys::Uint8Array = compressed[..].into();
        code_store
            .add(&value, Some(&JsValue::from_str(&digest)))
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_version(&self, id: &str) -> Result<Version, Error> {
        let transaction = self
            .database
            .transaction(&[VERSIONS], TransactionMode::ReadOnly)?;
        let store = transaction.object_store(VERSIONS)?;
        let id = JsValue::from_str(id);
        let stored = store.get(id).await?;
        let result = if let Some(stored) = stored {
            serde_wasm_bindgen::from_value(stored)?
        } else {
            Err(Error::NotFound)?
        };
        transaction.done().await?;
        Ok(result)
    }

    pub async fn get_code(&self, digest: &str) -> Result<String, Error> {
        let transaction = self
            .database
            .transaction(&[CODE], TransactionMode::ReadOnly)?;
        let store = transaction.object_store(CODE)?;
        let key = JsValue::from_str(digest);
        let Some(value) = store.get(key).await? else { return Err(Error::NotFound) };
        let Ok(array) = value.dyn_into::<js_sys::Uint8Array>() else { return Err(Error::BadData) };
        let vec = array.to_vec();
        let mut deflater = DeflateDecoder::new(vec.as_slice());
        let mut decompressed = String::new();
        if deflater.read_to_string(&mut decompressed).is_err() {
            return Err(Error::BadData);
        }
        transaction.done().await?;
        Ok(decompressed)
    }

    pub async fn list_versions(&self, scenario_name: &str) -> Result<Vec<Version>, Error> {
        let transaction = self
            .database
            .transaction(&[VERSIONS], TransactionMode::ReadOnly)?;
        let store = transaction.object_store(VERSIONS)?;
        let index = store.index("scenario_name")?;
        let scenario_name = JsValue::from_str(scenario_name);
        let query = Query::Key(scenario_name);
        let records = index.get_all(Some(query), None).await?;
        let mut result: Vec<_> = records
            .into_iter()
            .filter_map(|r| match serde_wasm_bindgen::from_value(r) {
                Ok(version) => Some(version),
                Err(e) => {
                    log::error!("Error deserializing version: {:?}", e);
                    None
                }
            })
            .collect();
        result.reverse();
        transaction.done().await?;
        Ok(result)
    }

    pub async fn check_digest_exists(&self, digest: &str) -> Result<bool, Error> {
        let transaction = self
            .database
            .transaction(&[VERSIONS], TransactionMode::ReadOnly)?;
        let store = transaction.object_store(VERSIONS)?;
        let index = store.index("digest")?;
        let digest = JsValue::from_str(digest);
        let query = Query::Key(digest);
        Ok(index.count(Some(query)).await.map(|count| count > 0)?)
    }

    pub async fn check_code_exists(&self, code: &str) -> Result<bool, Error> {
        let digest = digest(code);
        self.check_digest_exists(&digest).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub scenario_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub digest: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVersionParams {
    pub code: String,
    pub scenario_name: String,
    pub label: Option<String>,
}

fn digest(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IndexedDB error")]
    IndexedDBError(#[from] idb::Error),

    #[error("Serialization error")]
    SerializationError(#[from] serde_wasm_bindgen::Error),

    #[error("IO error")]
    IOError(#[from] std::io::Error),

    #[error("Not found")]
    NotFound,

    #[error("Bad data")]
    BadData,
}
