use mongodb::{
    bson::{oid::ObjectId, DateTime},
    Database,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum TeamType {
    Period,
    Semester,
}

#[derive(Debug, Deserialize)]
pub struct TeamModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub start_on: DateTime,
    pub team_type: TeamType,
    pub end_on: DateTime,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}
