use core::fmt;
use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::errors::MyError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TermType {
    Period,
    Semester,
}

impl fmt::Display for TermType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TermType::Period => write!(f, "Period "),
            TermType::Semester => write!(f, "Semester "),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TermModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub school: Option<ObjectId>,
    pub term_type: TermType,
    pub start_on: DateTime,
    pub end_on: DateTime,
    pub status: bool,
    pub children: Option<Vec<TermModel>>,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TermModelGet {
    pub id: String,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub school: Option<String>,
    pub term_type: TermType,
    pub start_on: String,
    pub status: bool,
    pub end_on: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TermModelNew {
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub school: Option<String>,
    pub term_type: TermType,
    pub start_on: String,
    pub end_on: String,
    pub status: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TermModelUpdate {
    pub name: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub school: Option<String>,
    pub term_type: Option<TermType>,
    pub start_on: Option<String>,
    pub end_on: Option<String>,
    pub status: Option<bool>,
    pub children: Option<Vec<TermModelUpdate>>,
}

impl TermModel {
    pub fn new(term: TermModelNew) -> Self {
        let start_on = DateTime::parse_rfc3339_str(&term.start_on)
            .map_err(|_| format!("Invalid date format for start_on: {}", term.start_on))
            .expect("Cannot parse date format");
        let end_on = DateTime::parse_rfc3339_str(&term.end_on)
            .map_err(|_| format!("Invalid date format for end_on: {}", term.end_on))
            .expect("Cannot parse date format");

        let school = term
            .school
            .as_deref() // Convert Option<String> to Option<&str>
            .map(|school_str| {
                ObjectId::from_str(school_str) // Try converting the &str to ObjectId
                    .map_err(|_| MyError::InvalidId) // Map the error to a custom error if needed
                    .expect("Cannot parse term school id")
            });

        TermModel {
            id: None,
            name: term.name,
            description: term.description,
            school,
            username: term.username,
            term_type: term.term_type,
            start_on,
            end_on,
            status: term.status,
            children: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}
