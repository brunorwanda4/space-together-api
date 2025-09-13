use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::object_id_serde;

use super::{address::Address, age::Age, gender::Gender, user_role::UserRole};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(
        rename = "_id",                         // Mongo expects "_id"
        alias = "id",                           // Accept "id" when deserializing JSON
        serialize_with = "object_id_serde::serialize",
        deserialize_with = "object_id_serde::deserialize"
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
    pub current_school_id: Option<String>,
    pub bio: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
