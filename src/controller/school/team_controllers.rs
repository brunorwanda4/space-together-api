use std::sync::Arc;

use mongodb::bson::Bson;

use crate::{
    error::team_error::{TeamError, TeamResult},
    models::school::team_model::{TeamModelGet, TeamModelNew},
    AppState,
};

pub async fn create_team_controller(
    team: TeamModelNew,
    school: Option<String>,
    db: Arc<AppState>,
) -> TeamResult<TeamModelGet> {
    let new = db
        .db
        .team_db
        .create_team(team)
        .await
        .map_err(|_| TeamError::CanNotCreateTeam);

    match new {
        Ok(result) => {
            let id = if let Bson::ObjectId(oid) = result.inserted_id {
                Ok(oid.to_hex())
            } else {
                Err(TeamError::CanNotCreateTeam)
            };
            let get = db.db.team_db.get_team_by_id(id.unwrap()).await;
            match get {
                Ok(result) => {
                    let get_team_model = TeamModelGet {
                        id: result
                            .id
                            .map_or("".to_string(), |obj_id| obj_id.to_string()),
                        name: result.name,
                        school: Some(
                            result
                                .school
                                .map_or("".to_string(), |obj_id| obj_id.to_string()),
                        ),
                        description: result.description,
                        end_on: result.end_on.to_rfc3339_string(),
                        start_on: result.start_on.to_rfc3339_string(),
                        team_type: result.team_type,
                        created_at: result.created_at.to_rfc3339_string(),
                        updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
                    };
                    Ok(get_team_model)
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
pub async fn get_team_controller(query: Arc<AppState>, id: String) -> TeamResult<TeamModelGet> {
    let get = query.db.team_db.get_team_by_id(id).await;
    match get {
        Ok(result) => {
            let get_team_model = TeamModelGet {
                id: result
                    .id
                    .map_or("".to_string(), |obj_id| obj_id.to_string()),
                name: result.name,
                school: Some(
                    result
                        .school
                        .map_or("".to_string(), |obj_id| obj_id.to_string()),
                ),
                description: result.description,
                end_on: result.end_on.to_rfc3339_string(),
                start_on: result.start_on.to_rfc3339_string(),
                team_type: result.team_type,
                created_at: result.created_at.to_rfc3339_string(),
                updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
            };
            Ok(get_team_model)
        }
        Err(err) => Err(err),
    }
}
