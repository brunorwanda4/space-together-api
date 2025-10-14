use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StudentStatus {
    #[default]
    Active,
    Suspended,
    Graduated,
    Left,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Student {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    // Connected user
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

    // Connected class
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub class_id: Option<ObjectId>,

    // Creator (school admin or system)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub creator_id: Option<ObjectId>,

    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<String>,
    pub date_of_birth: Option<String>, // You can change to DateTime<Utc> if you prefer

    pub registration_number: Option<String>,
    pub admission_year: Option<i32>,

    #[serde(default)]
    pub status: StudentStatus,

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
pub struct UpdateStudent {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<String>,
    pub date_of_birth: Option<String>,
    pub registration_number: Option<String>,
    pub admission_year: Option<i32>,
    pub status: Option<StudentStatus>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentWithRelations {
    #[serde(flatten)]
    pub student: Student,

    pub user: Option<crate::domain::user::User>,
    pub school: Option<crate::domain::school::School>,
    pub class: Option<crate::domain::class::Class>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkStudentIds {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateStudentStatus {
    pub ids: Vec<String>,
    pub status: StudentStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkStudentTags {
    pub ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PrepareStudentRequest {
    pub students: Vec<Student>,
    pub school_id: Option<String>,
    pub creator_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PrepareStudentsBulkRequest {
    pub students: Vec<Student>,
    pub school_id: Option<String>, // Optional: override school_id from token
    pub class_id: Option<String>,
    pub creator_id: Option<String>,
}
