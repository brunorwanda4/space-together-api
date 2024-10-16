use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

use super::student_model::StudentModel;

#[derive(Debug, Deserialize)]
struct ClassModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: ObjectId,
    pub name: String,
    pub students: Vec<StudentModel>,
    // teachers: Vec<TeacherModel>,
    pub reasons: Vec<String>,
    pub head_teacher: String,
}
