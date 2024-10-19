use core::fmt;

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
    pub trading_type: TradingType,
    pub schools_id: Option<Vec<ObjectId>>,
    pub education: EducationSystem,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradingModelUpdate {
    pub name: Option<String>,
    pub username: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub trading_type: Option<TradingType>,
    pub schools_id: Option<Vec<ObjectId>>,
    pub education: Option<EducationSystem>,
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
            reasons: None,
            is_active: false,
            education: trading.education,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    pub fn update(&mut self, updates: TradingModelUpdate) {
        let mut has_updates = false;

        if let Some(name) = updates.name {
            self.name = name;
            has_updates = true;
        }
        if let Some(username) = updates.username {
            self.username = username;
            has_updates = true;
        }
        if let Some(code) = updates.code {
            self.code = code;
            has_updates = true;
        }
        if let Some(description) = updates.description {
            self.description = Some(description);
            has_updates = true;
        }
        if let Some(trading_type) = updates.trading_type {
            self.trading_type = trading_type;
            has_updates = true;
        }
        if let Some(schools_id) = updates.schools_id {
            self.schools_id = Some(schools_id);
            has_updates = true;
        }
        if let Some(education) = updates.education {
            self.education = education;
            has_updates = true;
        }

        if has_updates {
            self.updated_at = Some(DateTime::now());
        }
    }
}
