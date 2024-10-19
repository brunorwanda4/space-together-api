use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use super::school_request_model::EducationSystem;

#[derive(Debug, Deserialize, Serialize)]
pub enum TradingType {
    Public,
    Private,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradingModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub code: String,
    pub description: Option<String>,
    pub trading_type: TradingType,
    pub schools_id: Option<Vec<ObjectId>>,
    pub reasons: Option<Vec<ObjectId>>,
    pub is_active: bool,
    pub education: EducationSystem,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradingModelNew {
    pub name: String,
    pub username: String,
    pub code: String,
    pub description: Option<String>,
    pub trading_type: TradingType,
    pub schools_id: Option<Vec<ObjectId>>,
    pub education: EducationSystem,
}

impl TradingModel {
    pub fn new(trading: TradingModelNew) -> Self {
        TradingModel {
            id: None,
            name: trading.name,
            username: trading.username,
            code: trading.code,
            description: trading.description,
            trading_type: trading.trading_type,
            schools_id: trading.schools_id,
            reasons: None,
            is_active: false,
            education: trading.education,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}
