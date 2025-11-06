use crate::{
    domain::{
        common_details::Image,
        user::{UpdateUserDto, User, UserStats},
    },
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

    /// ✅ Create a new user
    pub async fn create_user(&self, mut new_user: User) -> Result<User, String> {
        is_valid_name(&new_user.name)?;
        if let Some(ref username) = new_user.username {
            is_valid_username(username)?;
        }

        // 🔒 Uniqueness checks
        if let Ok(Some(_)) = self.repo.find_by_email(&new_user.email).await {
            return Err("Email already exists".to_string());
        }

        if let Some(ref username) = new_user.username {
            if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                return Err("Username already exists".to_string());
            }
        }

        // 🔑 Hash password
        if let Some(password) = new_user.password_hash.clone() {
            new_user.password_hash = Some(hash_password(&password));
        } else {
            return Err("Password is required".to_string());
        }

        // ☁️ Upload profile image
        if let Some(image_data) = new_user.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data).await?;
            new_user.image_id = Some(cloud_res.public_id);
            new_user.image = Some(cloud_res.secure_url);
        }

        // ☁️ Upload multiple background images
        if let Some(background_images_data) = new_user.background_images.clone() {
            let mut uploaded_images = Vec::new();
            for bg in background_images_data {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url).await?;
                uploaded_images.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            new_user.background_images = Some(uploaded_images);
        }

        let now = Some(Utc::now());
        new_user.created_at = now;
        new_user.updated_at = now;

        let inserted_user = self
            .repo
            .insert_user(&mut new_user)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_user(inserted_user))
    }

    /// ✅ Get user by ID
    pub async fn get_user_by_id(&self, id: &IdType) -> Result<User, String> {
        let user = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;
        Ok(sanitize_user(user))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User, String> {
        let user = self
            .repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;
        Ok(sanitize_user(user))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, String> {
        let user = self
            .repo
            .find_by_email(email)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;
        Ok(sanitize_user(user))
    }

    /// ✅ Update user (handles multiple background images)
    pub async fn update_user(
        &self,
        id: &IdType,
        mut updated_data: UpdateUserDto,
    ) -> Result<User, String> {
        let existing_user = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "User not found".to_string())?;

        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }

        // 🔒 Email uniqueness
        if let Some(ref email) = updated_data.email {
            if email != &existing_user.email {
                if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                    return Err("Email already exists".to_string());
                }
            }
        }

        // 🔒 Username uniqueness
        if let Some(ref username) = updated_data.username {
            if existing_user.username.as_deref() != Some(username.as_str()) {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("Username already exists".to_string());
                }
            }
        }

        // ☁️ Replace profile image
        if let Some(new_image_data) = updated_data.image.clone() {
            if Some(new_image_data.clone()) != existing_user.image {
                if let Some(old_image_id) = existing_user.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data).await?;
                updated_data.image_id = Some(cloud_res.public_id);
                updated_data.image = Some(cloud_res.secure_url);
            }
        }

        // ☁️ Replace or append background images
        if let Some(new_backgrounds) = updated_data.background_images.clone() {
            // Delete old backgrounds if any
            if let Some(old_bgs) = existing_user.background_images.clone() {
                for bg in old_bgs {
                    CloudinaryService::delete_from_cloudinary(&bg.id).await.ok();
                }
            }

            // Upload new ones
            let mut uploaded_bgs = Vec::new();
            for bg in new_backgrounds {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url).await?;
                uploaded_bgs.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            updated_data.background_images = Some(uploaded_bgs);
        }

        updated_data.updated_at = Some(Utc::now());

        let updated_user = self
            .repo
            .update_user_fields(&id.as_string(), &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_user(updated_user))
    }

    /// ✅ Delete user (deletes all images)
    pub async fn delete_user(&self, id: &IdType) -> Result<(), String> {
        if let Ok(Some(user)) = self.repo.find_by_id(id).await {
            if let Some(image_public_id) = user.image_id {
                CloudinaryService::delete_from_cloudinary(&image_public_id)
                    .await
                    .ok();
            }

            if let Some(backgrounds) = user.background_images {
                for bg in backgrounds {
                    CloudinaryService::delete_from_cloudinary(&bg.id).await.ok();
                }
            }
        }

        self.repo.delete_user(id).await.map_err(|e| e.message)
    }

    /// Add a school to a user
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

    /// Remove a school from a user
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

// 🧩 Helper for ID normalization
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
