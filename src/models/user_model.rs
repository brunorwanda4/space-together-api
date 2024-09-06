use std::{string, sync::{Arc, Mutex}};

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ModelsController{
    users : Arc<Mutex<Vec<Option<UserModel>>>>,
}

#[derive(Serialize , Deserialize , Debug)]
pub enum TGender {
    Male, FaMale , Other
}

impl TGender {
    pub(crate) fn to_string(&self) -> String{
        match self {
            TGender::Male => "Male".to_string(),
            TGender::FaMale => "Female".to_string(),
            TGender::Other => "Other".to_string(),
        }
    }
}

#[derive(Debug , Serialize , Deserialize)]
pub struct UserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id  : Option<ObjectId>,
    pub name : String,
    pub email : String,
    pub password : Option<String>,
    pub gender : Option<TGender>,
    pub image : Option<String>,
    pub birth_date : Option<DateTime>,
    pub facebook : Option<String>,
    pub twitter : Option<String>,
    pub instagram : Option<String>,
    pub linkedin : Option<String>,
    pub snapchat : Option<String>,
    pub whatsapp : Option<String>,
}