use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::trade::{Trade, TradeWithOthers},
    helpers::object_id_helpers,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainClass {
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
    pub username: String,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub trade_id: Option<ObjectId>,
    pub description: Option<String>,
    pub disable: Option<bool>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateMainClass {
    pub name: Option<String>,
    pub username: Option<String>,
    pub trade_id: Option<ObjectId>,
    pub description: Option<String>,
    pub disable: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainClassWithOthers {
    #[serde(flatten)]
    pub main_class: MainClass,

    #[serde(default)]
    pub trade: Option<TradeWithOthers>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainClassWithTrade {
    #[serde(flatten)]
    pub main_class: MainClass,

    #[serde(default)]
    pub trade: Option<Trade>,
}
