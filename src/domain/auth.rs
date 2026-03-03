use serde::{Deserialize, Serialize};

use crate::domain::common_details::UserRole;

#[derive(Deserialize)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub id: Option<String>,
    pub email: String,
    pub name: String,
    pub access_token: String,
    pub image: Option<String>,
    pub role: Option<UserRole>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub schools: Option<Vec<String>>,
    pub current_school_id: Option<String>,
    pub current_school_user_id: Option<String>,
    pub school_access_token: Option<String>,
}
