use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SchoolType {
    Primary,
    OLevel,
    ELevel,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchoolModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub school_request_id: ObjectId,
    pub students: Option<Vec<ObjectId>>,
    pub teachers: Option<Vec<ObjectId>>,
    pub staff: Option<Vec<ObjectId>>,
    pub headmaster: Vec<ObjectId>,
    pub school_type: Vec<SchoolType>,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}
