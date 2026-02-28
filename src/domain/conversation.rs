use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::RelatedUser, helpers::object_id_helpers, make_partial};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Conversation {
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

        pub participants: Vec<RelatedUser>,

        #[serde(default)]
        pub is_group: bool,

        pub name: Option<String>,

        #[serde(default = "default_key_version")]
        pub encryption_key_version: i32,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,
    } => ConversationPartial
}

fn default_key_version() -> i32 {
    1
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationKey {
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
    pub conversation_id: ObjectId,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub user_id: ObjectId,

    pub user_role: crate::domain::common_details::UserRole,

    pub encrypted_key_for_user: String,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}
