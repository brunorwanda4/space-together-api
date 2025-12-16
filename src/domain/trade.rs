use crate::{domain::sector::Sector, helpers::object_id_helpers, make_partial};
use chrono::{DateTime, Utc};
use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum TradeType {
    Senior,
    Primary,
    Level,
    Nursing,
    Other(String),
}

impl From<TradeType> for String {
    fn from(t: TradeType) -> String {
        match t {
            TradeType::Senior => "SENIOR".to_string(),
            TradeType::Primary => "PRIMARY".to_string(),
            TradeType::Level => "LEVEL".to_string(),
            TradeType::Nursing => "NURSING".to_string(),
            TradeType::Other(s) => s,
        }
    }
}

impl From<String> for TradeType {
    fn from(s: String) -> TradeType {
        match s.to_uppercase().as_str() {
            "SENIOR" => TradeType::Senior,
            "PRIMARY" => TradeType::Primary,
            "LEVEL" => TradeType::Level,
            "NURSING" => TradeType::Nursing,
            _ => TradeType::Other(s),
        }
    }
}

// ✅ Implement Display (this is what enables `.to_string()`)
impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeType::Senior => write!(f, "Senior"),
            TradeType::Primary => write!(f, "Primary"),
            TradeType::Level => write!(f, "Level"),
            TradeType::Nursing => write!(f, "Nursing"),
            TradeType::Other(s) => write!(f, "{s}"),
        }
    }
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
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
} => UpdateTrade
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
