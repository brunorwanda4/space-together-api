use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TopicsContent {
    pub name: String,
    pub description: String,
    pub content: Vec<String>,
    pub notes: Vec<ObjectId>,
    pub term: Option<ObjectId>,
}

#[derive(Debug, Deserialize)]
pub struct ReasonContent {
    pub topics: Vec<TopicsContent>,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ReasonModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub reason_content: ReasonContent,
    pub reason_code: String,
    pub severity: Option<String>,
    pub classes: Vec<ObjectId>,
    pub teachers_id: Vec<ObjectId>,
    pub updated_by: Option<Vec<ObjectId>>,
    pub model: Option<Vec<ObjectId>>,
    pub hours: u32,
    pub follow_up_required: bool,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
