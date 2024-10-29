use core::fmt;
use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum StaffRole {
    Headmaster,
    AssistantHeadmaster,
    Principal,
    DeputyPrincipal,
}

impl fmt::Display for StaffRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StaffRole::AssistantHeadmaster => write!(f, "AssistantHeadmaster"),
            StaffRole::Headmaster => write!(f, "Headmaster"),
            StaffRole::Principal => write!(f, "Principal"),
            StaffRole::DeputyPrincipal => write!(f, "DeputyPrincipal"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffSchoolModel {
    pub school_id: ObjectId,
    pub role: StaffRole,
    pub stated_job: DateTime,
    pub name: String,
    pub is_active: bool,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffSchoolModelNew {
    pub school_id: String,
    pub role: StaffRole,
    pub stated_job: String,
    pub name: String,
    pub is_active: bool,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub school: Option<Vec<StaffSchoolModel>>,
    pub school_active: Option<ObjectId>,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffModelNew {
    pub user_id: String,
    pub school: StaffSchoolModelNew,
}

impl StaffModel {
    pub fn new(staff: StaffModelNew) -> Self {
        StaffModel {
            id: None,
            created_at: DateTime::now(),
            updated_at: None,
            user_id: Some(
                ObjectId::from_str(&staff.user_id)
                    .expect("Can not convert user_id to String for staff model"),
            ),
            school_active: Some(
                ObjectId::from_str(&staff.school.school_id)
                    .expect("Can not convert school_id to String for staff model"),
            ),
            school: Some(
                [StaffSchoolModel {
                    school_id: ObjectId::from_str(&staff.school.school_id)
                        .expect("Can not convert school_id to String for staff model"),
                    name: staff.school.name.to_owned(),
                    stated_job: DateTime::now(), // TODO : TO CHANGE THE STATED_JOD TO STING
                    is_active: staff.school.is_active,
                    description: staff.school.description,
                    role: staff.school.role,
                }]
                .to_vec(),
            ),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffModelGet {
    pub id: String,
    pub user_id: String,
    pub school: Option<Vec<StaffSchoolModelNew>>,
    pub school_active: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}
