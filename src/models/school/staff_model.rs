use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StaffModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub role: Option<Vec<String>>,
    pub school: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}
