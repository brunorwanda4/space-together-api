use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    domain::{
        class::Class, class_subject::ClassSubject, common_details::Gender, school::School,
        user::User,
    },
    helpers::object_id_helpers,
    make_partial,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TeacherType {
    #[default]
    Regular, // Normal classroom teacher
    HeadTeacher,    // Manages a class or section
    SubjectTeacher, // Handles specific subjects
    Deputy,         // Assistant or deputy teacher
}
make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Teacher {
    #[serde(
        rename = "_id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    // Connected user (account)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub user_id: Option<ObjectId>,

    // Connected school
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
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
    pub image: Option<String>,
    pub image_id: Option<String>,
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
} =>UpdateTeacher
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeacherWithRelations {
    #[serde(flatten)]
    pub teacher: Teacher,

    pub user: Option<User>,
    pub creator: Option<User>,
    pub school: Option<School>,
    pub classes: Option<Vec<Class>>,
    pub subjects: Option<Vec<ClassSubject>>,
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

pub fn parse_teacher_type(type_str: &str) -> TeacherType {
    match type_str.to_lowercase().as_str() {
        "headteacher" => TeacherType::HeadTeacher,
        "subjectteacher" => TeacherType::SubjectTeacher,
        "deputy" => TeacherType::Deputy,
        _ => TeacherType::Regular,
    }
}
