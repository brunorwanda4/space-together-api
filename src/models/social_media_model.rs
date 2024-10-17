use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SocialMediaModel {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
}
