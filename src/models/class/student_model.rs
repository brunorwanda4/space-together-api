use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StudentRole {
    Student,
    ClassMonitor,
    ClassLeader,
    HeadBoy,
    HeadGirl,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudentModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub school: Vec<ObjectId>,
    pub role: StudentRole,
    pub class: Vec<ObjectId>,
}
