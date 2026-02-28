use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPublicKey {
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
    pub user_id: ObjectId,

    pub public_key: String, // PEM format

    #[serde(default = "default_key_algorithm")]
    pub key_algorithm: String, // "RSA-2048"

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

fn default_key_algorithm() -> String {
    "RSA-2048".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicKeyInfo {
    pub user_id: String,
    pub public_key: String,
    pub key_algorithm: String,
    pub created_at: DateTime<Utc>,
}
