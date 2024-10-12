use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::country_model::CountryModelLocation;

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
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolRequestModelNew {
    pub sended_by: String,
    pub name: String,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub location: CountryModelLocation,
    pub description: String,
}

impl SchoolRequestModel {
    pub fn new(request: SchoolRequestModelNew) -> Self {
        SchoolRequestModel {
            id: None,
            sended_by: ObjectId::parse_str(&request.sended_by).expect("Invalid School Request user id"),
            phone: request.phone,
            location: request.location,
            description: request.description,
            verified: Some(false),
            email: request.email,
            name: request.name,
            username: request.username,
        }
    }
}
