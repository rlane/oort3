use crate::project_id;
use anyhow::anyhow;
use chrono::Utc;
use firestore::*;
use oort_proto::{TournamentResults, TournamentSubmission};
use salvo::prelude::*;

async fn submit_tournament_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(&project_id()).await?;
    let payload = req.payload().await?;
    let mut obj: TournamentSubmission = serde_json::from_slice(payload)?;
    obj.timestamp = Utc::now();
    log::debug!("{:?}", obj);
    let docid = format!("{}.{}", obj.scenario_name, obj.userid);
    db.update_obj("tournament", &docid, &obj, None).await?;
    res.render(docid);
    Ok(())
}

#[handler]
pub async fn submit_tournament(req: &mut Request, res: &mut Response) {
    if let Err(e) = submit_tournament_internal(req, res).await {
        log::error!("error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string());
    }
}
async fn get_tournament_results_internal(
    req: &mut Request,
) -> anyhow::Result<salvo::writer::Json<TournamentResults>> {
    let db = FirestoreDb::new(&project_id()).await?;
    let id: String = req.param("id").ok_or(anyhow!("missing id parameter"))?;
    let tournament_results = db
        .get_obj::<TournamentResults>("tournament_results", &id)
        .await?;
    Ok(Json(tournament_results))
}

#[handler]
pub async fn get_tournament_results(req: &mut Request, res: &mut Response) {
    match get_tournament_results_internal(req).await {
        Ok(data) => res.render(data),
        Err(e) => {
            log::error!("error: {}", e);
            res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(e.to_string());
        }
    }
}
