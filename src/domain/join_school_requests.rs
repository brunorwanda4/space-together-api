use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JoinRole {
    Teacher,
    Student,
    Staff,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JoinStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinSchoolRequest {
    #[serde(
        rename = "_id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub id: ObjectId,

    // The school that sent the request
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub school_id: ObjectId,

    // The user receiving the invitation (could be teacher/student/staff)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub invited_user_id: ObjectId,

    // Role that the invited user will have in this school
    pub role: JoinRole,

    // Message or reason for joining (optional)
    pub message: Option<String>,

    // Request status
    pub status: JoinStatus,

    // When request was sent
    pub sent_at: DateTime<Utc>,

    // When it was accepted/rejected (optional)
    pub responded_at: Option<DateTime<Utc>>,

    // Who sent the invitation (usually school admin/staff)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub sent_by: ObjectId,

    // dates
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
