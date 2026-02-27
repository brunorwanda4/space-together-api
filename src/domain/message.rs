use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::{RelatedUser, UserRole}, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageSender {
    pub sender_role: UserRole,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub sender_id: ObjectId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MessageType {
    TEXT,
    FILE,
    SYSTEM,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
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
    pub conversation_id: ObjectId,

    pub sender: MessageSender,

    pub encrypted_payload: String,
    pub nonce: String,

    #[serde(default = "default_key_version")]
    pub key_version: i32,

    #[serde(default)]
    pub message_type: MessageType,

    pub file_url: Option<String>,
    pub file_public_id: Option<String>,

    #[serde(default)]
    pub read_by: Vec<RelatedUser>,

    pub client_message_id: String,

    pub deleted_at: Option<DateTime<Utc>>,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

fn default_key_version() -> i32 {
    1
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::TEXT
    }
}
