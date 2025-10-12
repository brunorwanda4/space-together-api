use crate::{
    domain::user::{UpdateUserDto, User, UserStats},
    errors::AppError,
    models::id_model::IdType,
    repositories::user_repo::UserRepo,
    services::cloudinary_service::CloudinaryService,
    utils::{
        hash::hash_password,
        names::{is_valid_name, is_valid_username},
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
    pub async fn get_all_users(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<User>, String> {
        let users = self
            .repo
            .get_all_users(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_users(users))
    }

    pub async fn get_user_stats(&self) -> Result<UserStats, String> {
        self.repo.get_user_stats().await.map_err(|e| e.message)
    }

    /// Create a new user
    pub async fn create_user(&self, mut new_user: User) -> Result<User, String> {
        is_valid_name(&new_user.name)?;
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

        // ✅ Handle image (can be base64 or path)
        if let Some(new_image_file) = new_user.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_file).await?;
            new_user.image_id = Some(cloud_res.public_id);
            new_user.image = Some(cloud_res.secure_url);
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
    pub async fn update_user(
        &self,
        id: &IdType,
        updated_data: UpdateUserDto,
    ) -> Result<User, String> {
        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }

        let mut user_to_update = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;

        // 🔒 Username uniqueness check
        if let Some(ref username) = updated_data.username {
            if user_to_update.username.as_ref() != Some(username) {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("username already exists".to_string());
                }
            }
        }

        // 🔒 Email uniqueness check
        if let Some(ref username) = updated_data.username {
            if user_to_update.username.as_deref() != Some(username.as_str()) {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("username already exists".to_string());
                }
            }
        }

        // ✅ Handle image
        if let Some(ref new_image) = updated_data.image {
            if let Some(old_image) = user_to_update.image_id.clone() {
                CloudinaryService::delete_from_cloudinary(&old_image)
                    .await
                    .ok();
            }
            let cloud_res = CloudinaryService::upload_to_cloudinary(new_image).await?;
            user_to_update.image_id = Some(cloud_res.public_id);
            user_to_update.image = Some(cloud_res.secure_url);
        }

        // ✅ Only overwrite if provided
        if let Some(name) = updated_data.name {
            user_to_update.name = name;
        }
        if let Some(email) = updated_data.email {
            user_to_update.email = email;
        }
        if let Some(username) = updated_data.username {
            user_to_update.username = Some(username);
        }
        if let Some(role) = updated_data.role {
            user_to_update.role = Some(role);
        }
        if let Some(phone) = updated_data.phone {
            user_to_update.phone = Some(phone);
        }
        if let Some(gender) = updated_data.gender {
            user_to_update.gender = Some(gender);
        }
        if let Some(age) = updated_data.age {
            user_to_update.age = Some(age);
        }
        if let Some(address) = updated_data.address {
            user_to_update.address = Some(address);
        }
        if let Some(bio) = updated_data.bio {
            user_to_update.bio = Some(bio);
        }
        if let Some(disable) = updated_data.disable {
            user_to_update.disable = Some(disable);
        }

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
        let user = self.repo.find_by_id(id).await.map_err(|e| e.message)?;

        if let Some(delete_user) = user {
            if let Some(image_public_id) = delete_user.image_id {
                CloudinaryService::delete_from_cloudinary(&image_public_id)
                    .await
                    .ok(); // ignore delete errors
            }
        }

        self.repo.delete_user(id).await.map_err(|e| e.message)
    }

    // ✅ Add school to user and set it as current_school_id
    pub async fn add_school_to_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, String> {
        let updated_user = self
            .repo
            .add_school_to_user(user_id, school_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_user(updated_user))
    }

    // ✅ Remove school from user (and clear current_school_id if it matches)
    pub async fn remove_school_from_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, String> {
        let updated_user = self
            .repo
            .remove_school_from_user(user_id, school_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_user(updated_user))
    }
}

/// (Optional) If frontend sends string IDs instead of ObjectId
fn extract_id_from_json_like(_user: &User) -> Option<String> {
    None
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
