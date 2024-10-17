use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;

use super::student_model::StudentModel;

#[derive(Debug, Deserialize)]
struct ClassModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub students: Vec<ObjectId>,
    pub trading: ObjectId,
    pub school: ObjectId,
    pub class_teacher: Vec<ObjectId>,
    pub class_monitor: ObjectId,
    pub class_leader: Option<ObjectId>,
    pub year: DateTime,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}
