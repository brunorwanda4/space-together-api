use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::country_model::CountryModelLocation;

#[derive(Debug , Clone ,Deserialize , Serialize)]
pub struct SchoolRequestModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id : Option<ObjectId>,
    pub name : String,
    pub username : String,
    pub email : String,
    pub phone : String,
    pub location : CountryModelLocation,
    pub description : String,
    pub verified : Option<bool>
}