use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub enum SchoolType {
    Public,
    Private,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub enum SchoolPrograms {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchoolModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub school_request: ObjectId,
    pub students: Option<Vec<ObjectId>>,
    pub teachers: Option<Vec<ObjectId>>,
    pub classes: Option<Vec<ObjectId>>,
    pub staff: Option<Vec<ObjectId>>,
    pub headmaster: Option<Vec<ObjectId>>,
    pub trading: Option<Vec<ObjectId>>,
    pub school_type: SchoolType,
    pub created_at: DateTime,
    pub updated_by: Option<Vec<ObjectId>>,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolModelNew {
    pub school_request: ObjectId,
    pub headmaster: ObjectId,
}

// impl SchoolModel {
//     pub fn new(school: SchoolModelNew) -> Self {
//         SchoolModel {
//             id: None,
//             school_request: school.school_request,
//             created_at: DateTime::now(),
//         }
//     }
// }
