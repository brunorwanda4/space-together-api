use core::fmt;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::models::country_model::CountryModelLocation;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SchoolType {
    Primary,
    OLevel,
    ELevel,
    Grades,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EducationSystem {
    TVET,
    REB,
}

impl fmt::Display for EducationSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EducationSystem::TVET => write!(f, "TVET"),
            EducationSystem::REB => write!(f, "REB"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SchoolRequestModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub sended_by: ObjectId,
    pub name: String,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub location: CountryModelLocation,
    pub description: String,
    pub logo: Option<String>,
    pub verified: Option<bool>,
    pub school_type: Vec<SchoolType>,
    pub education_system: Vec<EducationSystem>,
    pub school_id: Option<ObjectId>,
    pub is_private: bool,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolRequestModelNew {
    pub sended_by: String,
    pub name: String,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub logo: Option<String>,
    pub location: CountryModelLocation,
    pub education_system: Vec<EducationSystem>,
    pub description: String,
    pub is_private: bool,
    pub school_type: Vec<SchoolType>,
}

impl SchoolRequestModel {
    pub fn new(request: SchoolRequestModelNew) -> Self {
        SchoolRequestModel {
            id: None,
            sended_by: ObjectId::parse_str(&request.sended_by)
                .expect("Invalid School Request user id"),
            phone: request.phone,
            location: request.location,
            description: request.description,
            verified: Some(false),
            email: request.email,
            name: request.name,
            logo: request.logo,
            is_private: request.is_private,
            username: request.username,
            school_type: request.school_type,
            education_system: request.education_system,
            school_id: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolRequestModelGet {
    pub id: String,
    pub sended_by: String,
    pub name: String,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub description: String,
    pub location: CountryModelLocation,
    pub logo: Option<String>,
    pub verified: Option<bool>,
    pub school_type: Vec<SchoolType>,
    pub education_system: Vec<EducationSystem>,
    pub school_id: Option<String>,
    pub is_private: bool,
    pub created_at: String,
    pub updated_at: String,
}
