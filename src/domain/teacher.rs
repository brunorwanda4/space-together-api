use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    domain::{class::Class, common_details::Gender, school::School, subject::Subject, user::User},
    helpers::object_id_helpers,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TeacherType {
    #[default]
    Regular, // Normal classroom teacher
    HeadTeacher,    // Manages a class or section
    SubjectTeacher, // Handles specific subjects
    Deputy,         // Assistant or deputy teacher
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Teacher {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    // Connected user (account)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub user_id: Option<ObjectId>,

    // Connected school
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub school_id: Option<ObjectId>,

    // Creator (admin or system)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub creator_id: Option<ObjectId>,

    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,

    pub r#type: TeacherType, // Regular, HeadTeacher, etc.

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub class_ids: Option<Vec<ObjectId>>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub subject_ids: Option<Vec<ObjectId>>,

    #[serde(default)]
    pub is_active: bool,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct UpdateTeacher {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub r#type: Option<TeacherType>,
    pub class_ids: Option<Vec<ObjectId>>,
    pub subject_ids: Option<Vec<ObjectId>>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeacherWithRelations {
    #[serde(flatten)]
    pub teacher: Teacher,

    pub user: Option<User>,
    pub school: Option<School>,
    pub classes: Option<Vec<Class>>,
    pub subjects: Option<Vec<Subject>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkTeacherIds {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateTeacherActive {
    pub ids: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkTeacherTags {
    pub ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PrepareTeacherRequest {
    pub teachers: Vec<Teacher>,
    pub school_id: Option<String>,
    pub creator_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TeacherCountQuery {
    pub gender: Option<Gender>,
    pub teacher_type: Option<TeacherType>,
}

impl fmt::Display for TeacherType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TeacherType::Regular => "Regular",
                TeacherType::HeadTeacher => "HeadTeacher",
                TeacherType::SubjectTeacher => "SubjectTeacher",
                TeacherType::Deputy => "Deputy",
            }
        )
    }
}
