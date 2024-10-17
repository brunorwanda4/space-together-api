use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Province {
    name: String,
    districts: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CountryModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub provinces: Option<Vec<Province>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountryModelLocation {
    pub country: String,
    pub province: String,
    pub district: String,
    pub zip_code: Option<String>,
}
