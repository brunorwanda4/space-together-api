use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::UserRole, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditSeverity {
    INFO,
    WARNING,
    CRITICAL,
}

impl Default for AuditSeverity {
    fn default() -> Self {
        AuditSeverity::INFO
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLog {
    #[serde(
        rename = "_id",
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
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub user_id: ObjectId,

    pub user_role: UserRole,

    pub action: String,
    pub entity_type: String,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub entity_id: ObjectId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Document>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    #[serde(default)]
    pub severity: AuditSeverity,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLogWithRelations {
    #[serde(flatten)]
    pub audit_log: AuditLog,

    pub user: Option<crate::domain::user::User>,
    pub school: Option<crate::domain::school::School>,
}

#[derive(Debug, Clone)]
pub struct RequestMeta {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
