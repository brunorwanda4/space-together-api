use core::fmt;
use std::str::FromStr;

use crate::errors::MyError;

use super::school_request_model::EducationSystem;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TradingType {
    Public,
    Private,
}

impl fmt::Display for TradingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TradingType::Public => write!(f, "Public"),
            TradingType::Private => write!(f, "Private"),
        }
    }
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
    pub reasons: Option<Vec<String>>,
    pub trading_type: TradingType,
    pub schools_id: Option<Vec<ObjectId>>,
    pub education: EducationSystem,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradingModelUpdate {
    pub name: Option<String>,
    pub reasons: Option<Vec<String>>,
    pub username: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub trading_type: Option<TradingType>,
    pub schools_id: Option<Vec<String>>,
    pub education: Option<EducationSystem>,
    pub is_active: Option<bool>,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradingModelGet {
    pub id: String,
    pub name: String,
    pub username: String,
    pub code: String,
    pub description: Option<String>,
    pub trading_type: TradingType,
    pub schools_id: Option<Vec<String>>,
    pub reasons: Option<Vec<String>>,
    pub is_active: bool,
    pub education: EducationSystem,
    pub created_at: String,
    pub updated_at: Option<String>,
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
            reasons: Some(
                trading
                    .reasons
                    .unwrap_or_default()
                    .into_iter()
                    .map(|reason_str| {
                        ObjectId::from_str(&reason_str)
                            .map_err(|_| MyError::InvalidId)
                            .expect("Can not change trading id")
                    })
                    .collect::<Vec<ObjectId>>(),
            ),
            is_active: false,
            education: trading.education,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}
