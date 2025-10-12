use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Address {
    pub country: String,
    pub province: Option<String>,
    pub district: Option<String>,
    pub sector: Option<String>,
    pub cell: Option<String>,
    pub village: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub google_map_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contact {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub alt_phone: Option<String>,
    pub whatsapp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialMedia {
    pub platform: String, // e.g. "facebook", "twitter", "instagram"
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")] // "MALE", "FEMALE"
pub enum Gender {
    MALE,
    FEMALE,
    OTHER,
}
