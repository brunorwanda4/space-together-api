use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StaffModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub role: Option<Vec<String>>,
    pub school: Option<ObjectId>,
    pub name: Option<String>,
}
