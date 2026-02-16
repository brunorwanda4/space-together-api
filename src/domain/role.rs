use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RoleType {
    #[default]
    System,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PermissionScope {
    #[default]
    Own,
    Class,
    School,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Role {
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
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub school_id: Option<ObjectId>,

        pub name: String,
        pub description: Option<String>,

        #[serde(default)]
        pub role_type: RoleType,

        #[serde(default)]
        pub permissions: Vec<String>,

        #[serde(default)]
        pub is_active: bool,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,
    } => RolePartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleWithRelations {
    #[serde(flatten)]
    pub role: Role,

    pub school: Option<crate::domain::school::School>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    pub name: String,
    pub description: Option<String>,
    pub scope: PermissionScope,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRoleAssignment {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,
    pub role_id: ObjectId,
    pub school_id: ObjectId,

    #[serde(default = "Utc::now")]
    pub assigned_at: DateTime<Utc>,
}
