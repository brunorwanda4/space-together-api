use std::sync::Arc;

use mongodb::bson::Bson;

use crate::{
    error::team_error::{TeamError, TeamResult},
    models::school::team_model::TeamModelNew,
    AppState,
};

pub async fn create_team_controller(
    team: TeamModelNew,
    school: Option<String>,
    db: Arc<AppState>,
) -> TeamResult<String> {
    let new = db
        .db
        .team_db
        .create_team(team)
        .await
        .map_err(|_| TeamError::CanNotCreateTeam);

    match new {
        Ok(result) => {
            if let Bson::ObjectId(oid) = result.inserted_id {
                Ok(oid.to_hex()) // Convert ObjectId to hex string
            } else {
                Err(TeamError::CanNotCreateTeam)
            }
        }
        Err(err) => Err(err),
    }
}
