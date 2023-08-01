use crate::{project_id, Error};
use axum::extract::{Json, Path};
use chrono::Utc;
use firestore::*;
use oort_proto::{TournamentResults, TournamentSubmission};

pub async fn submit(Json(mut obj): Json<TournamentSubmission>) -> Result<String, Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    obj.timestamp = Utc::now();
    let docid = format!("{}.{}", obj.scenario_name, obj.userid);
    db.update_obj("tournament", &docid, &obj, None, None, None)
        .await?;
    Ok(docid)
}

pub async fn get_results(
    Path(id): Path<String>,
) -> Result<axum::response::Json<TournamentResults>, Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    let tournament_results = db
        .get_obj::<TournamentResults, _>("tournament_results", &id)
        .await?;
    Ok(Json(tournament_results))
}
