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

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub school_id: ObjectId,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        default
    )]
    pub invited_user_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        default
    )]
    pub class_id: Option<ObjectId>,

    pub role: JoinRole,
    pub email: String,
    pub r#type: String,
    pub message: Option<String>,

    pub status: JoinStatus,
    pub sent_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub sent_by: ObjectId,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JoinSchoolRequest {
    pub fn new(user: &User, school_id: &ObjectId, sent_by: &ObjectId) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            class_id: None,
            school_id: *school_id,
            invited_user_id: user.id,
            role: JoinRole::Student,
            email: user.email.clone(),
            r#type: user.role.clone().unwrap().to_string(),
            message: None,
            status: JoinStatus::Pending,
            sent_at: now,
            responded_at: None,
            expires_at: None,
            sent_by: sent_by.clone(),
            created_at: now,
            updated_at: now,
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinSchoolByCode {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinSchoolRequestResponseToken {
    pub school_token: String,
}
