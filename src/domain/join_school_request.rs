use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    domain::{class::Class, school::School, user::User},
    helpers::object_id_helpers,
};

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
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    // The school that sent the request
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub school_id: ObjectId,

    // The user receiving the invitation (could be teacher/student/staff)
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub invited_user_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize"
    )]
    pub class_id: Option<ObjectId>,

    // Role that the invited user will have in this school
    pub role: JoinRole,
    pub email: String,

    // user type role after created
    pub r#type: String, // if user is school staff: Director,HeadOfStudies. student: Active, teacher:  Regular, HeadTeacher, SubjectTeacher, Deputy

    // Message or reason for joining (optional)
    pub message: Option<String>,

    // Request status
    pub status: JoinStatus,

    // When request was sent
    pub sent_at: DateTime<Utc>,

    // When it was accepted/rejected (optional)
    pub responded_at: Option<DateTime<Utc>>,

    // When the request will expire (optional)
    pub expires_at: Option<DateTime<Utc>>,

    // Who sent the invitation (usually school admin/staff)
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub sent_by: ObjectId,

    // dates
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// This is the DTO you already showed, but with `pub` fields so it's usable cross-crate if needed.
/// (If you already have it in `domain/join_school_request.rs` you can skip redeclaring.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateJoinSchoolRequest {
    /// string IDs from client; repo/service will parse into ObjectId
    pub sent_by: String,
    pub email: String,
    pub role: JoinRole,
    pub r#type: String, // i was for get to add this
    pub school_id: String,
    pub message: Option<String>,
    pub class_id: Option<String>,
}

/// Bulk create wrapper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkCreateJoinSchoolRequest {
    pub requests: Vec<CreateJoinSchoolRequest>,
}

/// Payload to respond to a request (accept/reject/cancel)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespondToJoinRequest {
    /// the request id (string form)
    pub request_id: String,

    /// new status: Accepted | Rejected | Cancelled
    pub status: JoinStatus,

    /// who responded (string id of staff/admin); optional if server infers from auth
    pub responded_by: Option<String>,

    /// optional: when accepting, you may want to attach the invited_user_id (ObjectId string)
    pub invited_user_id: Option<String>,

    /// optional custom message
    pub message: Option<String>,
}

/// Payload to update the expiration date for a request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateRequestExpiration {
    pub request_id: String,
    /// ISO8601 datetime string expected; server should parse into chrono::DateTime<Utc>
    pub expires_at: DateTime<Utc>,
}

/// Bulk respond (accept/reject/cancel) by ids
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkRespondRequest {
    pub request_ids: Vec<String>,
    pub status: JoinStatus,
    pub responded_by: Option<String>,
    /// optional responded_at override
    pub responded_at: Option<DateTime<Utc>>,
}

/// Query/filter object for listing requests (used by handlers)
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct JoinRequestQuery {
    /// optional filter by email (exact or partial depending on handler)
    pub email: Option<String>,

    /// optional filter by school id (string)
    pub school_id: Option<String>,

    pub class_id: Option<String>,

    /// optional filter by status
    pub status: Option<JoinStatus>,

    /// optional filter by role
    pub role: Option<JoinRole>,

    /// pagination
    pub limit: Option<i64>,
    pub skip: Option<i64>,

    /// optional: only expired older than X days (useful for cleanup endpoints)
    pub older_than_days: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinSchoolRequestWithRelations {
    #[serde(flatten)]
    pub request: JoinSchoolRequest,

    #[serde(default)]
    pub school: Option<School>,

    #[serde(default)]
    pub class: Option<Class>,

    #[serde(default)]
    pub invited_user: Option<User>,

    #[serde(default)]
    pub sender: Option<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinRequestWithToken {
    #[serde(flatten)]
    pub request: JoinSchoolRequest,
    pub school_token: String,
}

impl fmt::Display for JoinRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JoinRole::Teacher => "Teacher",
                JoinRole::Student => "Student",
                JoinRole::Staff => "Staff",
            }
        )
    }
}
