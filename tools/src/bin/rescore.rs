use firestore::*;
use gcloud_sdk::google::firestore::v1::{value::ValueType, Document};
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct JsonMap {
    #[serde(flatten)]
    fields: HashMap<String, serde_json::Value>,
}

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("rescore=info"))
        .init();

    let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;

    const COLLECTION_NAME: &'static str = "telemetry";

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(COLLECTION_NAME.into()).with_filter(
                FirestoreQueryFilter::Composite(FirestoreQueryFilterComposite::new(vec![
                    FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                        "type".into(),
                        "FinishScenario".into(),
                    ))),
                    FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                        "success".into(),
                        true.into(),
                    ))),
                ])),
            ),
        )
        .await?;

    // (userid, scenario_name) -> (ticks, docid, code).
    let mut best_times: HashMap<(String, String), (i64, String, String)> = HashMap::new();

    let get_string = |doc: &Document, field: &str| {
        let v = doc.fields.get(field).unwrap();
        match &v.value_type {
            Some(ValueType::StringValue(s)) => s.clone(),
            _ => panic!(
                "Failed to get string field {:?} from document {:?}",
                field, doc.name
            ),
        }
    };

    let get_int = |doc: &Document, field: &str| {
        let v = doc.fields.get(field).unwrap();
        match v.value_type {
            Some(ValueType::IntegerValue(x)) => x,
            _ => panic!(
                "Failed to get integer field {:?} from document {:?}",
                field, doc.name
            ),
        }
    };

    let mut saved_docs = HashMap::new();

    for doc in &docs {
        let docid = doc.name.rsplit_once('/').unwrap().1.to_string();
        let userid = get_string(doc, "userid");
        let scenario_name = get_string(doc, "scenario_name");
        let ticks = get_int(doc, "ticks");
        let code = get_string(doc, "code");
        let key = (userid.clone(), scenario_name.clone());
        let update_best_times = if let Some((other_ticks, _docid, _code)) = best_times.get(&key) {
            ticks < *other_ticks
        } else {
            true
        };
        if update_best_times {
            best_times.insert(key, (ticks, docid.clone(), code.clone()));
        }
        saved_docs.insert(docid, doc);
    }

    let http_client = reqwest::Client::new();
    // docid, ticks
    let mut updates: Vec<(String, Option<u32>)> = Vec::new();

    for ((userid, scenario_name), (old_ticks, docid, code)) in best_times.iter() {
        log::info!(
            "Running simulations for userid={} scenario_name={} old_ticks={} docid={}",
            userid,
            scenario_name,
            old_ticks,
            docid
        );

        if let Some(wasm) = compile(&http_client, docid.into(), code.into()).await {
            log::info!("Successfully compiled to WASM");
            let status = run_simulations(scenario_name, wasm);
            match status {
                Some(new_ticks) => {
                    if *old_ticks as u32 != new_ticks {
                        log::info!("Updating ticks from {} to {}", old_ticks, new_ticks);
                        updates.push((docid.to_string(), Some(new_ticks)));
                    } else {
                        log::info!("Ticks unchanged");
                    }
                }
                None => {
                    log::warn!(
                        "Simulation failed for userid={} scenario_name={} docid={}",
                        userid,
                        scenario_name,
                        docid,
                    );
                    updates.push((docid.to_string(), None));
                }
            }
        }
    }

    log::info!("Applying {} updates", updates.len());

    for (docid, ticks) in updates {
        let doc = saved_docs.get(&docid).unwrap();
        let mut map = FirestoreDb::deserialize_doc_to::<JsonMap>(doc).unwrap();
        if let Some(ticks) = ticks {
            map.fields.insert("ticks".into(), ticks.into());
            map.fields.insert("success".into(), true.into());
            db.update_obj(
                "telemetry",
                &docid,
                &map,
                Some(vec!["ticks".into(), "success".into()]),
            )
            .await?;
        } else {
            map.fields.insert("ticks".into(), 0.into());
            map.fields.insert("success".into(), false.into());
            db.update_obj(
                "telemetry",
                &docid,
                &map,
                Some(vec!["ticks".into(), "success".into()]),
            )
            .await?;
        }
    }

    Ok(())
}

async fn compile(client: &reqwest::Client, docid: String, code: String) -> Option<Vec<u8>> {
    let url = "http://localhost:8081/compile";
    let result = client.post(url).body(code).send().await;
    let response = result.unwrap().error_for_status();
    match response {
        Ok(response) => Some(response.bytes().await.unwrap().as_ref().into()),
        Err(e) => {
            log::warn!("Failed to compile {:?}: {}", docid, e);
            None
        }
    }
}

fn run_simulations(scenario_name: &str, wasm: Vec<u8>) -> Option<u32> {
    let results: Vec<Option<u32>> = (0..10u32)
        .into_par_iter()
        .map(|seed| run_simulation(scenario_name, seed, wasm.clone()))
        .collect();
    log::info!("Results: {:?}", results);
    if results.iter().any(|x| x.is_none()) {
        return None;
    }
    Some(results.iter().map(|x| x.unwrap()).sum::<u32>() as u32 / results.len() as u32)
}

fn run_simulation(scenario_name: &str, seed: u32, wasm: Vec<u8>) -> Option<u32> {
    let scenario = scenario::load(scenario_name);
    let mut codes = scenario.initial_code();
    codes[0] = simulation::Code::Wasm(wasm);
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    match sim.status() {
        scenario::Status::Victory { team: 0 } => Some(sim.tick()),
        _ => None,
    }
}
