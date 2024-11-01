use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TeacherMOdel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub subject: String,
    pub school: Vec<ObjectId>,
    pub classes: Vec<ObjectId>,
}
