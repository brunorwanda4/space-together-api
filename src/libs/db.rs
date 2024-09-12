use std::{borrow::Borrow, env, str::FromStr, sync::{Arc, Mutex}, vec};
use axum::Json;
use chrono::Utc;
use lazy_regex::regex::Match;
use mongodb::{
    action::FindOne, bson::{doc, from_document, oid::ObjectId, to_bson, DateTime, Document}, error::Error, options::IndexOptions, results::{InsertOneResult, UpdateResult}, Client, Collection, IndexModel
};
use serde::{Deserialize, Serialize};

use crate::models::{images_models::{ProfileImageModel, ProfileImagesModel}, school::School, user_model::{UpdateUserModel, UserModel}};
use crate::errors::{Result , MyError};

#[derive(Debug , Clone,)]
pub struct Database {
    school : Collection<School>,
    user : Collection<UserModel>,
    profile_image : Collection<ProfileImagesModel>,
}

impl Database {
pub async fn init () -> Result<Self> {
    let uri: String = match env::var("MONGODB_URI") {
        Ok(val) => val.to_string(),
        Err(err) => "mongodb://localhost:27017/".to_string() 
    };

    let client: Client = Client::with_uri_str(uri).await.unwrap();
    let db = client.database("space-together");

    let image_db = client.database("space-together-images");
    let profile_image : Collection<ProfileImagesModel> = image_db.collection("profile-images");


    let school : Collection<School> = db.collection("schools");
    let user : Collection<UserModel> = db.collection("users");
   
   println!("✅ database is connected successfully");

   Ok(Self {
        school,
        user,
        profile_image
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
pub async fn create_user(&self, name: String, email: String, password: Option<String>) 
 -> Result<InsertOneResult> {
    let index_model = IndexModel::builder()
        .keys(doc! {
            "email" : 1,
            "username" : 1
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    self.user.create_index(index_model).await;
    
    let new_user = UserModel::new(name, email, password);

    let result = self
        .user
        .insert_one(new_user)
        .await;

    match result {
        Ok(ok) => Ok(ok),
        Err(err) => {
            println!("{}", err); // Log the error
            Err(MyError::CreateUserError) // Return the custom error
        },
    }

}

pub async fn get_user (&self , id : &str) -> Result<Json<UserModel>> {
    let user = self
    .user
    .find_one(doc! {"_id" : ObjectId::from_str(id).expect("Failed to find user by id")})
    .await;

    match user {
        Ok(Some (user)) => Ok(Json(user)),
        Ok(None) => Err(MyError::UserNotFound),
        Err(err) => Err(MyError::GetUserErr),
    }

}

pub async fn update_user(&self, id: &str, user: &UpdateUserModel) -> Result<UserModel> {
    // Convert id to ObjectId, return an error if it fails
    let obj_id = ObjectId::from_str(id).map_err(|_| MyError::InvalidUserId)?;
    let index_model = IndexModel::builder()
        .keys(doc! {"username" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    // Create the update document
    let mut update_doc = Document::new();
    if let Some(password) = &user.password {
        update_doc.insert("password", password);
    }

    if let Some(username) = &user.username {
        update_doc.insert("username", username);
    }
    if let Some(gender) = &user.gender {
        update_doc.insert("gender", gender.to_string());
    }
    if let Some(image) = &user.image {
        let user_object_id = Some(obj_id.clone());
        let now = DateTime::now().into();

        let new_image = ProfileImageModel {
            src : image.clone(),
            created_at : Some(DateTime::from_system_time(now))
        };

        let existing_profile_image = self
            .profile_image
            .find_one(doc! { "user_id": user_object_id })
            .await
            .ok()
            .expect("Couldn't find");

        if let Some(mut profile_images) =  existing_profile_image{
             if let Some(images) = &mut profile_images.images {
                    images.push(new_image.clone());
                } else {
                    profile_images.images = Some(vec![new_image.clone()]);
                }

            let new_image_bson = to_bson(&new_image).map_err(|_| MyError::DatabaseError)?;
            // update the existing profile image
            self.profile_image
                    .update_one(
                        doc! { "user_id": user_object_id },
                        doc! { "$push": { "images": new_image_bson}},  // Appending image directly
                    )
                    .await
                    .map_err(|_| MyError::DatabaseError)?;
        } else {
            let new_profile_images = ProfileImagesModel::new(image.to_string(), Some(id.to_string()));

            self.profile_image
                    .insert_one(new_profile_images)
                    .await
                    .map_err(|_| MyError::DatabaseError)?;
        }

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
    if let Some(phone_number) = &user.phone_number {
        update_doc.insert("phone_number", phone_number);
    }

    let now = Utc::now().into();

    update_doc.insert("updated_at", DateTime::from_system_time(now));

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
        Err(err) => {
            println!("Update user profile image error 😡: {}" , err);
            Err(MyError::DatabaseError)
        }, // Generic database error handling
    }
}

// no duplicate user

pub async fn get_user_by_email (&self , email : String) -> Result<UserModel> {
    let get_user = self
    .user
    .find_one(doc! {"email" : email})
    .await;

    match get_user {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(MyError::UserNotFound),
        Err(_) => Err(MyError::DatabaseError)
    }

}


}