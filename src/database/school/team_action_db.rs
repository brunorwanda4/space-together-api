use std::str::FromStr;

use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::results::InsertOneResult;
use mongodb::Collection;

use crate::error::team_error::{TeamError, TeamResult};
use crate::models::school::team_model::{TeamModel, TeamModelGet, TeamModelNew};

#[derive(Debug, Clone)]
pub struct TeamActionDb {
    pub team: Collection<TeamModel>,
}

impl TeamActionDb {
    pub async fn create_team(&self, team: TeamModelNew) -> TeamResult<InsertOneResult> {
        let new = TeamModel::new(team);
        let new_team = self
            .team
            .insert_one(new)
            .await
            .map_err(|_| TeamError::CanNotCreateTeam);

        match new_team {
            Ok(result) => Ok(result),
            Err(_) => Err(TeamError::CanNotCreateTeam),
        }
    }
    pub async fn get_team_by_id(&self, id: String) -> TeamResult<TeamModel> {
        let obj_id = ObjectId::from_str(&id)
            .map_err(|_| TeamError::TeamInvalidId)
            .expect("Can not invalid id");
        let get = self
            .team
            .find_one(doc! {"_id" : obj_id})
            .await
            .map_err(|_| TeamError::CanNotGetTeam);

        match get {
            Ok(Some(team)) => Ok(team),
            Ok(None) => Err(TeamError::TeamNotFound),
            Err(_) => Err(TeamError::CanNotGetTeam),
        }
    }
}
