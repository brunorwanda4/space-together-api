use mongodb::results::InsertOneResult;
use mongodb::Collection;

use crate::error::team_error::{TeamError, TeamResult};
use crate::models::school::team_model::{TeamModel, TeamModelNew};

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
}
