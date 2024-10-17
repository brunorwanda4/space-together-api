use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;

use super::school_model::SchoolType;

#[derive(Debug, Deserialize)]
pub struct TradingModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub trading_type: SchoolType,
    pub schools_id: Option<Vec<ObjectId>>,
    pub reasons: Option<Vec<ObjectId>>,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}
pub struct TradingNew {
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub trading_type: SchoolType,
    pub schools_id: Option<Vec<ObjectId>>,
}
