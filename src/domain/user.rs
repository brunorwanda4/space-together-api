use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::object_id_serde;

use super::{address::Address, age::Age, gender::Gender, user_role::UserRole};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_serde::serialize",
        deserialize_with = "object_id_serde::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    pub name: String,
    pub email: String,
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,

    pub role: Option<UserRole>,
    pub image: Option<String>,
    pub phone: Option<String>,

    pub gender: Option<Gender>,
    pub age: Option<Age>,
    pub address: Option<Address>,

    #[serde(
        serialize_with = "object_id_serde::serialize",
        deserialize_with = "object_id_serde::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub current_school_id: Option<ObjectId>,

    pub bio: Option<String>,

    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}
