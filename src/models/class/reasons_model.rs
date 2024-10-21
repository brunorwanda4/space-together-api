use core::fmt;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ReasonType {
    Specific,
    General,
    Complementary,
}

impl fmt::Display for ReasonType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReasonType::Specific => write!(f, "Specific"),
            ReasonType::General => write!(f, "General"),
            ReasonType::Complementary => write!(f, "Complementary"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TopicsContent {
    pub name: String,
    pub description: String,
    pub content: Vec<String>,
    pub notes: Option<Vec<ObjectId>>,
    pub term: Option<ObjectId>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReasonContent {
    pub topics: Vec<TopicsContent>,
    pub description: String,
    pub notes: Option<Vec<ObjectId>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReasonModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub code: String,
    pub reason_content: Option<ReasonContent>,
    pub severity: Option<String>,
    pub classes: Option<Vec<ObjectId>>,
    pub teachers_id: Option<Vec<ObjectId>>,
    pub updated_by: Option<Vec<ObjectId>>,
    pub trading: Option<Vec<ObjectId>>,
    pub hours: u32,
    pub follow_up_required: bool,
    pub school: Option<ObjectId>,
    pub is_public: bool,
    pub reason_type: ReasonType,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReasonModelGet {
    pub id: String,
    pub name: String,
    pub code: String,
    pub severity: Option<String>,
    pub classes: Option<Vec<String>>,
    pub teachers_id: Option<Vec<String>>,
    pub updated_by: Option<Vec<String>>,
    pub reason_content: Option<ReasonContent>,
    pub trading: Option<Vec<String>>,
    pub hours: u32,
    pub follow_up_required: bool,
    pub school: Option<String>,
    pub is_public: bool,
    pub reason_type: ReasonType,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReasonModelNew {
    pub name: String,
    pub reason_content: Option<ReasonContent>,
    pub code: String,
    pub severity: Option<String>,
    pub classes: Option<Vec<ObjectId>>,
    pub teachers_id: Option<Vec<ObjectId>>,
    pub updated_by: Option<Vec<ObjectId>>,
    pub trading: Option<Vec<ObjectId>>,
    pub hours: u32,
    pub follow_up_required: bool,
    pub school: Option<ObjectId>,
    pub is_public: bool,
    pub reason_type: ReasonType,
    pub is_active: bool,
}

impl ReasonModel {
    pub fn new(reason: ReasonModelNew) -> Self {
        ReasonModel {
            id: None,
            name: reason.name,
            code: reason.code,
            reason_content: None,
            severity: reason.severity,
            classes: None,
            teachers_id: None,
            updated_by: None,
            trading: reason.trading,
            hours: reason.hours,
            follow_up_required: reason.follow_up_required,
            school: reason.school,
            is_public: reason.is_public,
            reason_type: reason.reason_type,
            is_active: reason.is_active,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}
