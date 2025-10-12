use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::Gender, helpers::object_id_helpers};

use super::{age::Age, common_details::Address, user_role::UserRole};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
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
    // cloudinary
    pub image_id: Option<String>,
    pub image: Option<String>,

    pub phone: Option<String>,

    pub gender: Option<Gender>,
    pub age: Option<Age>,
    pub address: Option<Address>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub current_school_id: Option<ObjectId>,

    pub bio: Option<String>,
    pub disable: Option<bool>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub schools: Option<Vec<ObjectId>>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub accessible_classes: Option<Vec<ObjectId>>,

    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub role: Option<UserRole>,
    pub image: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub age: Option<Age>,
    pub address: Option<Address>,
    pub bio: Option<String>,
    pub disable: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct UserStats {
    pub total: i64,
    pub male: i64,
    pub female: i64,
    pub other: i64,
    pub admins: i64,
    pub staff: i64,
    pub students: i64,
    pub teachers: i64,
    pub assigned_school: i64,
    pub no_school: i64,
    pub recent_30_days: i64,
}
