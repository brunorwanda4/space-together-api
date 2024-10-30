use std::time::SystemTime;

use bcrypt::hash;
use chrono::Utc;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use super::images_models::ProfileImageModel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TGender {
    Male,
    Female,
    Other,
}

impl TGender {
    pub(crate) fn to_string(&self) -> String {
        match self {
            TGender::Male => "Male".to_string(),
            TGender::Female => "Female".to_string(),
            TGender::Other => "Other".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TUserType {
    Teacher,
    Student,
    Parent,
    Staff,
    Directer,
}

impl TUserType {
    pub(crate) fn to_string(&self) -> String {
        match self {
            TUserType::Teacher => "Teacher".to_string(),
            TUserType::Directer => "Directer".to_string(),
            TUserType::Student => "Student".to_string(),
            TUserType::Parent => "Parent".to_string(),
            TUserType::Staff => "Staff".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ProfileImageType {
    ObjectId(ObjectId),
    String(String),
    Images(Vec<ProfileImageModel>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub gender: Option<TGender>,
    pub image: Option<ProfileImageType>,
    pub birth_date: Option<String>,
    pub facebook: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub snapchat: Option<String>,
    pub whatsapp: Option<String>,
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub user_type: Option<TUserType>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

impl UserModel {
    pub fn new(name: String, email: String, password: Option<String>) -> Self {
        let hashed_password = password.as_ref().map(|pw| hash(pw, 10).unwrap());

        let now: SystemTime = Utc::now().into();
        UserModel {
            id: None,
            name,
            email,
            password: hashed_password,
            gender: None,
            image: None,
            birth_date: None,
            facebook: None,
            twitter: None,
            instagram: None,
            linkedin: None,
            snapchat: None,
            whatsapp: None,
            username: None,
            phone_number: None,
            user_type: None,
            created_at: Some(DateTime::from_system_time(now)),
            updated_at: Some(DateTime::from_system_time(now)),
        }
    }
}

// Request model for creating a user
#[derive(Deserialize)]
pub struct CreateUserRequestModel {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub gender: Option<TGender>,
    pub image: Option<String>,
    pub birth_date: Option<String>,
    pub facebook: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub snapchat: Option<String>,
    pub whatsapp: Option<String>,
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub user_type: Option<TUserType>,
}

// login

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUserModel {
    pub email: String,
    pub password: String,
}
