use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::common_details::{Address, Contact, SocialMedia},
    helpers::object_id_helpers,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolType {
    Public,
    Private,
    Charter,
    International,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AffiliationType {
    Government,
    Religious,
    NGO,
    Independent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SchoolMemberType {
    Mixed,
    Boys,
    Girls,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AttendanceSystemType {
    Manual,
    Online,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct School {
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
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub creator_id: Option<ObjectId>,

    // Basic info
    pub username: String,
    pub logo: Option<String>,
    pub logo_id: Option<String>,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub school_type: Option<SchoolType>, // e.g. public, private, boarding

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub curriculum: Option<Vec<ObjectId>>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub education_level: Option<Vec<ObjectId>>, // e.g. primary, secondary, university
    pub accreditation_number: Option<String>,
    pub affiliation: Option<AffiliationType>,

    // Members (can later expand this into its own struct)
    pub school_members: Option<SchoolMemberType>,

    // Location
    pub address: Option<Address>,
    pub contact: Option<Contact>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,

    // Students
    pub student_capacity: Option<i32>,
    pub uniform_required: Option<bool>,
    pub attendance_system: Option<AttendanceSystemType>, // manual, biometric, RFID, etc.
    pub scholarship_available: Option<bool>,

    // Facilities
    pub classrooms: Option<i32>,
    pub library: Option<bool>,
    pub labs: Option<Vec<String>>,
    pub sports_extracurricular: Option<Vec<String>>,
    pub online_classes: Option<bool>,

    pub database_name: Option<String>,
    pub is_active: Option<bool>,
    // Meta
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSchool {
    pub username: Option<String>,
    pub logo: Option<String>,
    pub logo_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub school_type: Option<SchoolType>,
    pub curriculum: Option<Vec<ObjectId>>,
    pub education_level: Option<Vec<ObjectId>>,
    pub accreditation_number: Option<String>,
    pub affiliation: Option<AffiliationType>,
    pub school_members: Option<SchoolMemberType>,
    pub address: Option<Address>,
    pub contact: Option<Contact>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,
    pub student_capacity: Option<i32>,
    pub uniform_required: Option<bool>,
    pub attendance_system: Option<AttendanceSystemType>,
    pub scholarship_available: Option<bool>,
    pub classrooms: Option<i32>,
    pub library: Option<bool>,
    pub labs: Option<Vec<String>>,
    pub sports_extracurricular: Option<Vec<String>>,
    pub online_classes: Option<bool>,
    pub database_name: Option<String>,
    pub is_active: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolStats {
    pub total: i64,
    pub public: i64,
    pub private: i64,
    pub active: i64,
    pub inactive: i64,
    pub recent_30_days: i64,
}
