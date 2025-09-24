use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::sector::Sector, models::object_id_serde};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_serde::serialize",
        deserialize_with = "object_id_serde::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_serde::serialize",
        deserialize_with = "object_id_serde::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub sector_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_serde::serialize",
        deserialize_with = "object_id_serde::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub trade_id: Option<ObjectId>, // self relation

    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub class_min: i32,
    pub class_max: i32,
    pub r#type: String, // Senior, Primary, Level, Nursing
    pub disable: Option<bool>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTrade {
    pub sector_id: Option<ObjectId>,
    pub trade_id: Option<ObjectId>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub class_min: Option<i32>,
    pub class_max: Option<i32>,
    pub r#type: Option<String>, // Senior, Primary, Level, Nursing
    pub disable: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeWithSector {
    #[serde(flatten)]
    pub trade: Trade,

    #[serde(default)]
    pub sector: Option<Sector>,
}
