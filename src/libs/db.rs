use std::{env, str::FromStr, sync::{Arc, Mutex}};
use axum::Json;
use mongodb::{
    action::FindOne, bson::{doc, from_document, oid::ObjectId, Document}, error::Error, results::{InsertOneResult, UpdateResult}, Client, Collection
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
    let db = client.database("SpaceTogetherLocal");

    let image_db = client.database("images");


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

pub async fn update_user(&self, id: &str, user: &UserModel) -> Result<UserModel> {
    // Convert id to ObjectId, return an error if it fails
    let obj_id = ObjectId::from_str(id).map_err(|_| MyError::InvalidUserId)?;

    // Create the update document
    let mut update_doc = Document::new();
    if let Some(password) = &user.password {
        update_doc.insert("password", password);
    }
    if let Some(gender) = &user.gender {
        update_doc.insert("gender", gender.to_string());
    }
    if let Some(image) = &user.image {
        update_doc.insert("image", image);
    }
    if let Some(birth_date) = &user.birth_date {
        update_doc.insert("birth_date", birth_date);
    }
    if let Some(facebook) = &user.facebook {
        update_doc.insert("facebook", facebook);
    }
    if let Some(twitter) = &user.twitter {
        update_doc.insert("twitter", twitter);
    }
    if let Some(instagram) = &user.instagram {
        update_doc.insert("instagram", instagram);
    }
    if let Some(linkedin) = &user.linkedin {
        update_doc.insert("linkedin", linkedin);
    }
    if let Some(snapchat) = &user.snapchat {
        update_doc.insert("snapchat", snapchat);
    }
    if let Some(whatsapp) = &user.whatsapp {
        update_doc.insert("whatsapp", whatsapp);
    }

    // Attempt to update the user
    let update_res = self
        .user
        .find_one_and_update(
            doc! { "_id": obj_id },
            doc! { "$set": update_doc },
        )
        .await;

    // Handle possible outcomes of the update
    match update_res {
        Ok(Some(updated_user)) => Ok(updated_user), // Return the updated user
        Ok(None) => Err(MyError::UserNotFound), // Handle case where user was not found
        Err(_) => Err(MyError::DatabaseError), // Generic database error handling
    }
}

// no duplicate user

pub async fn get_user_by_email (&self , email : String) -> Result<Json<UserModel>> {
    let user = self
    .user
    .find_one(doc! {"email" : email})
    .await
    .ok()
    .expect("can not find as  user by email");

    if(user.is_none()) {
        return Err(MyError::UserNotFound);
    }

    let user_json: Json<UserModel> = Json(user.unwrap());
    Ok(user_json)
}


}