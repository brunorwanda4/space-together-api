use axum::Json;
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, DateTime, Document},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};
use std::{str::FromStr, vec};

use crate::errors::{MyError, Result};
use crate::models::{
    images_models::{ProfileImageModel, ProfileImagesModel},
    user_model::{ProfileImageType, UpdateUserModel, UserModel},
};

#[derive(Debug, Clone)]
pub struct UserActionDb {
    pub user: Collection<UserModel>,
    pub avatar: Collection<ProfileImagesModel>,
}

impl UserActionDb {
    pub async fn create_user(
        &self,
        name: String,
        email: String,
        password: Option<String>,
    ) -> Result<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "email" : 1,
                "username" : 1
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let one_index = self.user.create_index(index).await;
        if one_index.is_err() {
            return Err(MyError::UserCanNotCreate);
        }

        let new_user = UserModel::new(name, email, password);

        let result = self.user.insert_one(new_user).await;

        match result {
            Ok(ok) => Ok(ok),
            Err(err) => {
                println!("{}", err); // Log the error
                Err(MyError::CreateUserError) // Return the custom error
            }
        }
    }

    pub async fn get_user(&self, id: &str) -> Result<Json<UserModel>> {
        // Convert the string ID to an ObjectId
        let user_id = ObjectId::parse_str(id).map_err(|_| MyError::InvalidUserId)?;

        let user_result = self
            .user
            .find_one(doc! { "_id": user_id })
            .await
            .map_err(|_| MyError::DatabaseError)?;

        // Step 2: Check if user exists
        let mut user = match user_result {
            Some(u) => u,
            None => return Err(MyError::UserNotFound),
        };

        let mut profile_images: Vec<ProfileImageModel> = Vec::new();

        if let Some(image) = user.image.take() {
            if let ProfileImageType::ObjectId(image_id) = image {
                // Fetch profile image(s) from `profile_image_collection`
                let image_result = self
                    .avatar
                    .find_one(doc! { "_id": image_id })
                    .await
                    .map_err(|_| MyError::CanNotFindImage)?;

                if let Some(profile_images_model) = image_result {
                    if let Some(images) = profile_images_model.images {
                        // Assign all the `ProfileImageModel` objects to `profile_images`
                        profile_images = images;
                    }
                }
                user.image = Some(ProfileImageType::Images(profile_images));
            } else if let ProfileImageType::String(image_string) = image {
                user.image = Some(ProfileImageType::String(image_string));
            }
        }

        Ok(Json(user))
    }

    pub async fn update_user(&self, id: &str, user: &UpdateUserModel) -> Result<UserModel> {
        // Convert id to ObjectId, return an error if it fails
        let obj_id = ObjectId::from_str(id).map_err(|_| MyError::InvalidUserId)?;
        let index = IndexModel::builder()
            .keys(doc! {"username" : 1})
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let one_index = self.avatar.create_index(index).await;

        if one_index.is_err() {
            return Err(MyError::AvatarUserIdIsReadyExit);
        }

        // Create the update document
        let mut update_doc = Document::new();

        if let Some(username) = &user.username {
            update_doc.insert("username", username);
        }
        if let Some(gender) = &user.gender {
            update_doc.insert("gender", gender.to_string());
        }

        if let Some(user_type) = &user.user_type {
            update_doc.insert("user_type", user_type.to_string());
        }

        if let Some(image) = &user.image {
            let user_object_id = Some(obj_id);
            let now = DateTime::now().into();

            let new_image = ProfileImageModel {
                src: image.clone(),
                created_at: Some(DateTime::from_system_time(now)),
            };

            let existing_profile_image = self
                .avatar
                .find_one(doc! { "user_id": user_object_id })
                .await
                .expect("Couldn't find user");

            if let Some(mut profile_images) = existing_profile_image {
                if let Some(images) = &mut profile_images.images {
                    images.push(new_image.clone());
                } else {
                    profile_images.images = Some(vec![new_image.clone()]);
                }

                let new_image_bson = to_bson(&new_image).map_err(|_| MyError::DatabaseError)?;
                // update the existing profile image
                self.avatar
                    .update_one(
                        doc! { "user_id": user_object_id },
                        doc! { "$push": { "images": new_image_bson}}, // Appending image directly
                    )
                    .await
                    .map_err(|_| MyError::DatabaseError)?;
            } else {
                let new_profile_images =
                    ProfileImagesModel::new(image.to_string(), Some(id.to_string()));

                let create_new_image = self
                    .avatar
                    .insert_one(new_profile_images)
                    .await
                    .map_err(|_| MyError::DatabaseError)?;

                update_doc.insert(
                    "image",
                    create_new_image
                        .inserted_id
                        .as_object_id()
                        .expect("can't insert "),
                );
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
            .find_one_and_update(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await;

        // Handle possible outcomes of the update
        match update_res {
            Ok(Some(ok)) => Ok(ok),                 // Return the updated user
            Ok(None) => Err(MyError::UserNotFound), // Handle case where user was not found
            Err(err) => {
                println!("Update user profile image error 😡: {}", err);
                Err(MyError::DatabaseError)
            } // Generic database error handling
        }
    }

    // no duplicate user

    pub async fn get_user_by_email(&self, email: String) -> Result<UserModel> {
        let get_user = self.user.find_one(doc! {"email" : email}).await;

        match get_user {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(MyError::UserNotFound),
            Err(_) => Err(MyError::DatabaseError),
        }
    }

    pub async fn get_user_by_username(&self, username: Option<String>) -> Result<UserModel> {
        let get_user = self.user.find_one(doc! {"username": username}).await;

        match get_user {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(MyError::UserNotFound),
            Err(_) => Err(MyError::DatabaseError),
        }
    }
}
