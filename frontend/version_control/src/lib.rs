use idb::{Database, Error, Factory, KeyPath, ObjectStoreParams, Query, TransactionMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

const SCHEMA_VERSION: u32 = 2;
const VERSIONS: &str = "versions";

pub struct VersionControl {
    pub database: Database,
}

impl VersionControl {
    pub async fn new() -> Result<VersionControl, Error> {
        let factory = Factory::new()?;
        let mut open_request = factory.open("oort_version_control", Some(SCHEMA_VERSION)).unwrap();

        open_request.on_upgrade_needed(|event| {
            let database = event.database().unwrap();
            let transaction = event.transaction().unwrap().unwrap();

            if !database.store_names().contains(&VERSIONS.to_string()) {
                let mut store_params = ObjectStoreParams::new();
                store_params.key_path(Some(KeyPath::new_single("id")));
                database
                    .create_object_store(VERSIONS, store_params)
                    .unwrap();
            }

            let store = transaction
                .object_store(VERSIONS)
                .unwrap();

            if !store.index_names().contains(&"scenario_name".to_string()) {
                store
                    .create_index("scenario_name", KeyPath::new_single("scenario_name"), None)
                    .unwrap();
            }

            if !store.index_names().contains(&"digest".to_string()) {
                store
                    .create_index("digest", KeyPath::new_single("digest"), None)
                    .unwrap();
            }
        });

        let database = open_request.await?;
        Ok(VersionControl { database })
    }

    pub async fn create_version(&self, params: &CreateVersionParams) -> Result<(), Error> {
        let timestamp = chrono::Utc::now();
        let timestamp_string = timestamp.format("%Y%m%d-%H%M%S");
        let digest = digest(&params.code);
        let id = format!("{}-{}", timestamp_string, digest);
        let version = Version {
            id,
            code: params.code.clone(),
            scenario_name: params.scenario_name.clone(),
            timestamp,
            digest,
            label: params.label.clone(),
        };
        let transaction = self
            .database
            .transaction(&[VERSIONS], TransactionMode::ReadWrite)?;
        let store = transaction.object_store(VERSIONS).unwrap();
        store
            .add(&serde_wasm_bindgen::to_value(&version).unwrap(), None)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_version(&self, id: &str) -> Result<Option<Version>, Error> {
        let transaction = self
            .database
            .transaction(&[VERSIONS], TransactionMode::ReadOnly)
            .unwrap();
        let store = transaction.object_store(VERSIONS).unwrap();
        let id = JsValue::from_str(id);
        let stored = store.get(id).await?;
        let result: Option<Version> = stored.map(|v| serde_wasm_bindgen::from_value(v).unwrap());
        transaction.done().await?;
        Ok(result)
    }

    pub async fn list_versions(&self, scenario_name: &str) -> Result<Vec<Version>, Error> {
        let transaction = self
            .database
            .transaction(&[VERSIONS], TransactionMode::ReadOnly)
            .unwrap();
        let store = transaction.object_store(VERSIONS).unwrap();
        let index = store.index("scenario_name").unwrap();
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
            .transaction(&[VERSIONS], TransactionMode::ReadOnly)
            .unwrap();
        let store = transaction.object_store(VERSIONS).unwrap();
        let index = store.index("digest").unwrap();
        let digest = JsValue::from_str(digest);
        let query = Query::Key(digest);
        index.count(Some(query)).await.map(|count| count > 0)
    }

    pub async fn check_code_exists(&self, code: &str) -> Result<bool, Error> {
        let digest = digest(code);
        self.check_digest_exists(&digest).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub code: String,
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
