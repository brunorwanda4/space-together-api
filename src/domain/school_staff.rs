use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum SchoolStaffType {
    #[default]
    Director,
    HeadOfStudies,
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolStaff {
    #[serde(
        rename = "_id",
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
} => SchoolStaffPartial
}

pub fn parse_staff_type(type_str: &str) -> SchoolStaffType {
    match type_str.to_lowercase().as_str() {
        "director" => SchoolStaffType::Director,
        "headofstudies" => SchoolStaffType::HeadOfStudies,
        _ => SchoolStaffType::HeadOfStudies,
    }
}
