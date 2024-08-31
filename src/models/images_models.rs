use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug , Serialize , Deserialize)]
pub struct ProfileImageModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id : Option<ObjectId>,
    pub user_id : Option<ObjectId> ,
    pub src : Option<String>,
}