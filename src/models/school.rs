use std::sync::Arc;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug , Deserialize , Serialize)]
pub enum SchoolCategories {
    Online,
    Primary,
    Middle,
    Vocational,
    Homeschooling,
    Boarding,
    TVET,
    HighSchool,
    International,
}

#[derive(Debug , Deserialize , Serialize)]
pub struct School {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id : Option<ObjectId>,
    pub name : String,
    pub username : String,
    pub logo : Option<String>,
    pub county : String,
    pub province : String,
    pub city : String,
    pub email : String,
    pub website : Option<String>,
    pub twitter : Option<String>,
    pub facebook : Option<String>,
    pub whatsapp : Option<String>,
    pub category : Option<Vec<SchoolCategories>>,
    pub description : String,
    pub images : Option<Vec<String>>,
    pub created_by : ObjectId,
    pub updated_by : Option<ObjectId>,
    pub created_at : DateTime,
    pub updated_at : DateTime,
}

