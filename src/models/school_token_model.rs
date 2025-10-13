use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::school::{AffiliationType, SchoolType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolToken {
    pub id: String,
    pub creator_id: Option<String>,

    pub name: String,
    pub username: String,

    pub logo: Option<String>,

    pub school_type: Option<SchoolType>, // or SchoolType if you want enum in token
    pub affiliation: Option<AffiliationType>, // optional string form of affiliation

    pub database_name: String,

    pub created_at: Option<DateTime<Utc>>,

    pub exp: usize,
    pub iat: usize,
}
