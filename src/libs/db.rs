use std::{env, str::FromStr, sync::{Arc, Mutex}};
use axum::Json;
use mongodb::{
    action::FindOne, bson::{doc, from_document, oid::ObjectId, Document}, results::InsertOneResult, Client, Collection,error::Error
};
use serde::{Deserialize, Serialize};

use crate::models::{school::School, user_model::UserModel};
use crate::errors::{Result , MyError};

#[derive(Debug , Clone,)]
pub struct Database {
    school : Collection<School>,
    user : Collection<UserModel>,
}

impl Database {
pub async fn init () -> Result<Self> {
    let uri: String = match env::var("MONGODB_URI") {
        Ok(val) => val.to_string(),
        Err(err) => "mongodb://localhost:27017/".to_string() 
    };

    let client: Client = Client::with_uri_str(uri).await.unwrap();
    let db: mongodb::Database = client.database("SpaceTogetherLocal");

    let school : Collection<School> = db.collection("schools");
    let user : Collection<UserModel> = db.collection("users");
   
   println!("✅ database is connected successfully");

   Ok(Self {
        school,
        user,
   })
}

pub async fn create_school(&self , school : School) -> Result<InsertOneResult> {
    let res = self
    .school
    .insert_one(school)
    .await
    .ok()
    .expect("can't create a new school");
    Ok(res)
}

// ----- user CRUD  ----------
pub async fn create_user(&self , user : UserModel) -> Result<InsertOneResult> {
    let res = self
    .user
    .insert_one(user)
    .await
    .ok()
    .expect("can't create a new user");

    Ok(res)
}

pub async fn get_user (&self , id : &str) -> Result<Json<UserModel>> {
    let user = self
    .user
    .find_one(doc! {"_id" : ObjectId::from_str(id).expect("Failed to find user by id")})
    .await
    .ok()
    .expect("can not find a user");

    let user_json : Json<UserModel> = Json(user.unwrap());

    Ok(user_json)
}

// pub async fn get_users(&self ,) -> Result<Vec<UserModel>> {
//     let res = self
//     .user
//     .find(doc! {})
//     .await
//     .ok()
//     .expect("can't find a user");

//     let mut users : Vec<UserModel> = Vec::new();

//     while let Some(res) = res.try_next().await?{
//         match res {
//             Ok(doc) => {
//                 let my_user : UserModel = from_document(doc).expect("Error converting documents");
//                 users.push(my_user);
//             }
//             Err(err) => panic!("Error converting user document: {}", err),
//         }
//     }
//     Ok(users)
// }


}