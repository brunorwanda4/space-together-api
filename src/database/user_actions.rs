// src/database/user_actions.rs

use std::str::FromStr;

use mongodb::{bson::{doc, oid::ObjectId, DateTime, to_bson, Document}, results::InsertOneResult, options::IndexOptions, IndexModel};
use serde::{Serialize, Deserialize};
use axum::Json;
use crate::errors::{Result, MyError};
use crate::models::user_model::{UpdateUserModel, UserModel};
use crate::models::images_models::ProfileImagesModel;
use mongodb::Collection;

pub struct UserActions {
    pub user: Collection<UserModel>,
    pub profile_image: Collection<ProfileImagesModel>,
}

impl UserActions {
    pub async fn create_user(&self, name: String, email: String, password: Option<String>) -> Result<InsertOneResult> {
        let index_model = IndexModel::builder()
            .keys(doc! {
                "email": 1,
                "username": 1
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.user.create_index(index_model).await;

        let new_user = UserModel::new(name, email, password);

        let result = self.user.insert_one(new_user).await;

        match result {
            Ok(ok) => Ok(ok),
            Err(_) => Err(MyError::CreateUserError),
        }
    }

    pub async fn get_user(&self, id: &str) -> Result<Json<UserModel>> {
        let obj_id = ObjectId::from_str(id).map_err(|_| MyError::InvalidUserId)?;

        let user = self.user.find_one(doc! { "_id": obj_id }).await;

        match user {
            Ok(Some(user)) => Ok(Json(user)),
            Ok(None) => Err(MyError::UserNotFound),
            Err(_) => Err(MyError::GetUserErr),
        }
    }

    pub async fn update_user(&self, id: &str, user: &UpdateUserModel) -> Result<UserModel> {
        let obj_id = ObjectId::from_str(id).map_err(|_| MyError::InvalidUserId)?;

        let mut update_doc = Document::new();
        if let Some(password) = &user.password {
            update_doc.insert("password", password);
        }
        if let Some(username) = &user.username {
            update_doc.insert("username", username);
        }
        // Additional user fields update...
        let now = DateTime::now().into();
        update_doc.insert("updated_at", DateTime::from_system_time(now));

        let update_res = self
            .user
            .find_one_and_update(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await;

        match update_res {
            Ok(Some(updated_user)) => Ok(updated_user),
            Ok(None) => Err(MyError::UserNotFound),
            Err(_) => Err(MyError::DatabaseError),
        }
    }

    pub async fn get_user_by_email(&self, email: String) -> Result<UserModel> {
        let get_user = self.user.find_one(doc! { "email": email }).await;

        match get_user {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(MyError::UserNotFound),
            Err(_) => Err(MyError::DatabaseError),
        }
    }
}
