use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::errors::MyError;

#[derive(Debug, Deserialize, Serialize)]
pub enum TeamType {
    Period,
    Semester,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub school: Option<ObjectId>,
    pub team_type: TeamType,
    pub start_on: DateTime,
    pub end_on: DateTime,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamModelNew {
    pub name: String,
    pub description: Option<String>,
    pub school: Option<String>,
    pub team_type: TeamType,
    pub start_on: String,
    pub end_on: String,
}

impl TeamModel {
    pub fn new(team: TeamModelNew) -> Self {
        let start_on = DateTime::parse_rfc3339_str(&team.start_on)
            .map_err(|_| format!("Invalid date format for start_on: {}", team.start_on))
            .expect("Cannot parse date format");
        let end_on = DateTime::parse_rfc3339_str(&team.end_on)
            .map_err(|_| format!("Invalid date format for end_on: {}", team.end_on))
            .expect("Cannot parse date format");

        let school = team
            .school
            .as_deref() // Convert Option<String> to Option<&str>
            .map(|school_str| {
                ObjectId::from_str(school_str) // Try converting the &str to ObjectId
                    .map_err(|_| MyError::InvalidId) // Map the error to a custom error if needed
                    .expect("Cannot parse team school id")
            });

        TeamModel {
            id: None,
            name: team.name,
            description: team.description,
            school,
            team_type: team.team_type,
            start_on,
            end_on,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}
