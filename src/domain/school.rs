use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::common_details::{Address, Contact, SocialMedia},
    helpers::object_id_helpers,
};

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
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub school_type: Option<String>, // e.g. public, private, boarding
    pub curriculum: Option<Vec<String>>,
    pub education_level: Option<Vec<String>>, // e.g. primary, secondary, university
    pub accreditation_number: Option<String>,
    pub affiliation: Option<String>,

    // Members (can later expand this into its own struct)
    pub school_members: Option<Vec<String>>, // store user IDs or role names

    // Location
    pub address: Option<Address>,
    pub contact: Option<Contact>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,

    // Students
    pub student_capacity: Option<i32>,
    pub uniform_required: Option<bool>,
    pub attendance_system: Option<String>, // manual, biometric, RFID, etc.
    pub scholarship_available: Option<bool>,

    // Facilities
    pub classrooms: Option<i32>,
    pub library: Option<bool>,
    pub labs: Option<Vec<String>>,
    pub sports_extracurricular: Option<Vec<String>>,
    pub online_classes: Option<bool>,

    // Meta
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSchool {
    pub username: Option<String>,
    pub logo: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub school_type: Option<String>,
    pub curriculum: Option<Vec<String>>,
    pub education_level: Option<Vec<String>>,
    pub accreditation_number: Option<String>,
    pub affiliation: Option<String>,
    pub school_members: Option<Vec<String>>,
    pub address: Option<Address>,
    pub contact: Option<Contact>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,
    pub student_capacity: Option<i32>,
    pub uniform_required: Option<bool>,
    pub attendance_system: Option<String>,
    pub scholarship_available: Option<bool>,
    pub classrooms: Option<i32>,
    pub library: Option<bool>,
    pub labs: Option<Vec<String>>,
    pub sports_extracurricular: Option<Vec<String>>,
    pub online_classes: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}
