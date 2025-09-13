use crate::{
    domain::user::User,
    errors::AppError,
    models::id_model::IdType,
    repositories::user_repo::UserRepo,
    utils::{
        hash::hash_password,
        names::is_valid_username,
        user_utils::{sanitize_user, sanitize_users},
    },
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct UserService<'a> {
    repo: &'a UserRepo,
}

impl<'a> UserService<'a> {
    pub fn new(repo: &'a UserRepo) -> Self {
        Self { repo }
    }

    /// Get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>, String> {
        let users = self.repo.get_all_users().await.map_err(|e| e.message)?;
        Ok(sanitize_users(users))
    }

    pub async fn create_user(&self, mut new_user: User) -> Result<User, String> {
        // Validate username if provided
        if let Some(ref username) = new_user.username {
            is_valid_username(username)?;
        }

        if let Ok(Some(_)) = self.repo.find_by_email(&new_user.email).await {
            return Err("Email already exists".to_string());
        }

        if let Some(ref username) = new_user.username {
            if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                return Err("username already exists".to_string());
            }
        }

        // Hash password if provided
        if let Some(password) = new_user.password_hash.clone() {
            new_user.password_hash = Some(hash_password(&password));
        } else {
            return Err("Password is required".to_string());
        }

        // Set timestamps
        let now = Some(Utc::now());
        new_user.created_at = now;
        new_user.updated_at = now;

        new_user.id = Some(ObjectId::new());

        // Save user in database
        let inserted_user = self
            .repo
            .insert_user(&new_user)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_user(inserted_user))
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, id: &IdType) -> Result<User, String> {
        let user = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;

        Ok(sanitize_user(user))
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<User, String> {
        let user = self
            .repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;

        Ok(sanitize_user(user))
    }

    /// Update a user by id
    pub async fn update_user(&self, id: &IdType, updated_data: User) -> Result<User, String> {
        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }

        let mut user_to_update = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;

        if user_to_update.username != updated_data.username {
            if let Some(ref username) = updated_data.username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("username already exists".to_string());
                }
            }
        }

        if user_to_update.email != updated_data.email {
            if let Ok(Some(_)) = self.repo.find_by_email(&updated_data.email).await {
                return Err("Email already exists".to_string());
            }
        }

        // Overwrite fields if provided
        user_to_update.name = updated_data.name;
        user_to_update.email = updated_data.email;
        user_to_update.username = updated_data.username;
        if let Some(password) = updated_data.password_hash {
            user_to_update.password_hash = Some(hash_password(&password));
        }
        user_to_update.role = updated_data.role.or(user_to_update.role);
        user_to_update.phone = updated_data.phone.or(user_to_update.phone);
        user_to_update.image = updated_data.image.or(user_to_update.image);
        user_to_update.gender = updated_data.gender.or(user_to_update.gender);
        user_to_update.age = updated_data.age.or(user_to_update.age);
        user_to_update.address = updated_data.address.or(user_to_update.address);
        user_to_update.current_school_id = updated_data
            .current_school_id
            .or(user_to_update.current_school_id);
        user_to_update.bio = updated_data.bio.or(user_to_update.bio);
        user_to_update.updated_at = Some(Utc::now());

        let updated_user = self
            .repo
            .update_user(id, &user_to_update)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_user(updated_user))
    }

    /// Delete a user by id
    pub async fn delete_user(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_user(id).await.map_err(|e| e.message)
    }
}

fn extract_id_from_json_like(_user: &User) -> Option<String> {
    None // <- fill this if your JSON comes as string instead of ObjectId
}

pub fn normalize_user_ids(mut user: User) -> Result<User, AppError> {
    if let Some(school_id) = user.current_school_id.take() {
        user.current_school_id = Some(school_id);
    } else if let Some(raw) = extract_id_from_json_like(&user) {
        match ObjectId::parse_str(&raw) {
            Ok(oid) => user.current_school_id = Some(oid),
            Err(_) => {
                return Err(AppError {
                    message: format!("Invalid ObjectId string for current_school_id: {}", raw),
                })
            }
        }
    }
    Ok(user)
}
