use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ModelsController{
    users : Arc<Mutex<Vec<Option<UserModel>>>>,
}

#[derive(Debug , Serialize , Deserialize)]
pub struct UserModel {
    pub name : String,
    pub password : String,
}