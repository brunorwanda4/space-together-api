use serde::{Deserialize, Serialize};

use crate::domain::{gender::Gender, user_role::UserRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub image: Option<String>,
    pub phone: Option<String>,
    pub role: Option<UserRole>,
    pub gender: Option<Gender>,
    pub current_school_id: Option<String>,
    pub iat: Option<i64>, // issued at
    pub exp: Option<i64>, // expiration
}
