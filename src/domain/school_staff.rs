use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum SchoolStaffType {
    #[default]
    Director,
    HeadOfStudies,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolStaff {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub user_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub school_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub creator_id: Option<ObjectId>,

    pub name: String,
    pub email: String,

    pub r#type: SchoolStaffType, // Director, HeadOfStudies

    #[serde(default)]
    pub is_active: bool,

    #[serde(default)]
    pub tags: Vec<String>,

    // Department or office where the user works (e.g., "Administration", "Finance", "Library", "IT", "HR")
    // pub department: Option<String>,

    // Job title or position (e.g., "Accountant", "Secretary", "Clerk", "Librarian", "Security")
    // pub job_title: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct UpdateSchoolStaff {
    pub name: Option<String>,
    pub email: Option<String>,
    pub r#type: Option<SchoolStaffType>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolStaffWithRelations {
    #[serde(flatten)]
    pub staff: SchoolStaff,

    // Optionally embed related user or school info
    pub user: Option<crate::domain::user::User>,
    pub school: Option<crate::domain::school::School>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkIdsRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateActiveStatusRequest {
    pub ids: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkTagsRequest {
    pub ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PrepareStaffRequest {
    pub staff_members: Vec<SchoolStaff>,
    pub school_id: Option<String>,
    pub creator_id: Option<String>,
}

impl fmt::Display for SchoolStaffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SchoolStaffType::Director => "Director",
                SchoolStaffType::HeadOfStudies => "HeadOfStudies",
            }
        )
    }
}

pub fn parse_staff_type(type_str: &str) -> SchoolStaffType {
    match type_str.to_lowercase().as_str() {
        "director" => SchoolStaffType::Director,
        "headofstudies" => SchoolStaffType::HeadOfStudies,
        _ => SchoolStaffType::HeadOfStudies,
    }
}
