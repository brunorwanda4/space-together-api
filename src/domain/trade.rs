use chrono::{DateTime, Utc};
use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{domain::sector::Sector, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TradeType {
    Senior,
    Primary,
    Level,
    Nursing,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
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
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub sector_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub trade_id: Option<ObjectId>, // self relation

    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub class_min: i32,
    pub class_max: i32,
    pub r#type: TradeType,
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
    pub r#type: Option<TradeType>, // Senior, Primary, Level, Nursing
    pub disable: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeWithParent {
    #[serde(flatten)]
    pub trade: Trade,

    #[serde(default)]
    pub sector: Option<Sector>,
}

fn deserialize_parent_trade<'de, D>(
    deserializer: D,
) -> Result<Option<Box<TradeWithParent>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = bson::Document::deserialize(deserializer)?;
    if val.is_empty() {
        Ok(None)
    } else {
        bson::from_document(val)
            .map(Box::new)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeWithOthers {
    #[serde(flatten)]
    pub trade: Trade,

    #[serde(default)]
    pub sector: Option<Sector>,

    #[serde(default, deserialize_with = "deserialize_parent_trade")]
    pub parent_trade: Option<Box<TradeWithParent>>,
}
