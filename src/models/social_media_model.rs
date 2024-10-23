use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SocialMediaModel {
    pub name: String,
    pub url: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContactModel {
    pub phone: String,
    pub email: String,
    pub social_media: Vec<SocialMediaModel>,
    pub address: Option<String>,
    pub website: Option<String>,
}
