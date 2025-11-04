use crate::{
    domain::{
        class::{Class, ClassWithOthers, ClassWithSchool, UpdateClass},
        common_details::Image,
    },
    models::id_model::IdType,
    repositories::class_repo::ClassRepo,
    services::cloudinary_service::CloudinaryService,
    utils::{
        class_utils::{sanitize_class, sanitize_classes},
        code::generate_code,
        names::is_valid_username,
    },
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct ClassService<'a> {
    repo: &'a ClassRepo,
}

impl<'a> ClassService<'a> {
    pub fn new(repo: &'a ClassRepo) -> Self {
        Self { repo }
    }

    /// Get all classes
    pub async fn get_all_classes(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .get_all_classes(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(classes))
    }

    /// Get all classes with school information
    pub async fn get_all_classes_with_school(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithSchool>, String> {
        self.repo
            .get_all_with_school(filter, limit, skip)
            .await
            .map_err(|e| e.message)
    }

    /// Get active classes
    pub async fn get_active_classes(&self) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .get_active_classes()
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(classes))
    }

    /// Create a new class
    pub async fn create_class(&self, mut new_class: Class) -> Result<Class, String> {
        is_valid_username(&new_class.username)?;

        // Check if code already exists
        if let Some(class_code) = &new_class.code {
            if let Ok(Some(_)) = self.repo.find_by_code(class_code).await {
                return Err("Class code already exists".to_string());
            }
        }

        if let Some(image_data) = new_class.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data).await?;
            new_class.image_id = Some(cloud_res.public_id);
            new_class.image = Some(cloud_res.secure_url);
        }

        // ☁️ Upload multiple background images
        if let Some(background_images_data) = new_class.background_images.clone() {
            let mut uploaded_images = Vec::new();
            for bg in background_images_data {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url).await?;
                uploaded_images.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            new_class.background_images = Some(uploaded_images);
        }

        // Check if username already exists
        if let Ok(Some(_)) = self.repo.find_by_username(&new_class.username).await {
            return Err("Class username already exists".to_string());
        }

        // Generate class code if not provided
        if new_class.code.is_none() {
            new_class.code = Some(generate_code());
        }

        // Set timestamps
        let now = Utc::now();
        new_class.created_at = now;
        new_class.updated_at = now;

        // Set default values for optional fields
        if !new_class.is_active {
            new_class.is_active = true;
        }

        let class_id = ObjectId::new();
        new_class.id = Some(class_id);

        // Save class in database
        let inserted_class = self
            .repo
            .insert_class(&new_class)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_class(inserted_class))
    }

    /// Get class by ID
    pub async fn get_class_by_id(&self, id: &IdType) -> Result<Class, String> {
        let class = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        Ok(sanitize_class(class))
    }

    /// Get class by ID with related information
    pub async fn get_class_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<ClassWithOthers, String> {
        let class = self
            .repo
            .find_by_id_with_others(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        Ok(class)
    }

    /// Get class by username
    pub async fn get_class_by_username(&self, username: &str) -> Result<Class, String> {
        let class = self
            .repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        Ok(sanitize_class(class))
    }

    /// Get class by username with related information
    pub async fn get_class_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<ClassWithOthers, String> {
        let class = self
            .repo
            .find_by_username_with_others(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        Ok(class)
    }

    /// Get class by code
    pub async fn get_class_by_code(&self, code: &str) -> Result<Class, String> {
        let class = self
            .repo
            .find_by_code(code)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        Ok(sanitize_class(class))
    }

    /// Get class by code with related information
    pub async fn get_class_by_code_with_others(
        &self,
        code: &str,
    ) -> Result<ClassWithOthers, String> {
        let class = self
            .repo
            .find_by_code_with_others(code)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        Ok(class)
    }

    /// Get classes by school ID
    pub async fn get_classes_by_school_id(&self, school_id: &IdType) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .find_by_school_id(school_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(classes))
    }

    /// Get classes by creator ID
    pub async fn get_classes_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .find_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(classes))
    }

    /// Get classes by class teacher ID
    pub async fn get_classes_by_class_teacher_id(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .find_by_class_teacher_id(teacher_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(classes))
    }

    /// Get classes by main class ID
    pub async fn get_classes_by_main_class_id(
        &self,
        main_class_id: &IdType,
    ) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .find_by_main_class_id(main_class_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(classes))
    }

    pub async fn update_class_merged(
        &self,
        id: &IdType,
        updated_data: UpdateClass,
    ) -> Result<Class, String> {
        // ✅ Validate username
        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }

        let mut existing_class = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        // ✅ Unique username check
        if let Some(ref username) = updated_data.username {
            if existing_class.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("Class username already exists".into());
                }
            }
        }

        // ✅ Unique code check
        if let Some(ref code) = updated_data.code {
            if existing_class.code.as_ref() != code.as_ref() {
                if let Ok(Some(_)) = self
                    .repo
                    .find_by_code(code.as_ref().unwrap_or(&"".to_string()))
                    .await
                {
                    return Err("Class code already exists".into());
                }
            }
        }

        // ✅ Merge fields
        if let Some(v) = updated_data.name {
            existing_class.name = v;
        }
        if let Some(v) = updated_data.username {
            existing_class.username = v;
        }
        if let Some(v) = updated_data.code {
            existing_class.code = v;
        }
        if let Some(v) = updated_data.school_id {
            existing_class.school_id = v;
        }
        if let Some(v) = updated_data.class_teacher_id {
            existing_class.class_teacher_id = v;
        }
        if let Some(v) = updated_data.r#type {
            existing_class.r#type = v;
        }
        if let Some(v) = updated_data.is_active {
            existing_class.is_active = v;
        }
        if let Some(v) = updated_data.description {
            existing_class.description = v;
        }
        if let Some(v) = updated_data.capacity {
            existing_class.capacity = Some(v);
        }
        if let Some(v) = updated_data.subject {
            existing_class.subject = v;
        }
        if let Some(v) = updated_data.grade_level {
            existing_class.grade_level = v;
        }
        if let Some(v) = updated_data.tags {
            existing_class.tags = v;
        }

        // ✅ Handle image replacement
        if let Some(new_image_data) = updated_data.image.clone() {
            if existing_class.image != Some(new_image_data.clone()) {
                if let Some(old_image_id) = existing_class.image_id.clone() {
                    let _ = CloudinaryService::delete_from_cloudinary(&old_image_id).await;
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data)
                    .await
                    .map_err(|e| format!("Cloud upload failed: {}", e))?;
                existing_class.image_id = Some(cloud_res.public_id);
                existing_class.image = Some(cloud_res.secure_url);
            }
        }

        // ✅ Handle background images
        if let Some(new_backgrounds) = updated_data.background_images.clone() {
            if let Some(old_bgs) = existing_class.background_images.clone() {
                for bg in old_bgs {
                    let _ = CloudinaryService::delete_from_cloudinary(&bg.id).await;
                }
            }

            let mut uploaded_bgs = Vec::new();
            for bg in new_backgrounds {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url)
                    .await
                    .map_err(|e| format!("Failed to upload background image: {}", e))?;
                uploaded_bgs.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            existing_class.background_images = Some(uploaded_bgs);
        }

        existing_class.updated_at = Utc::now();

        // ✅ Build final UpdateClass
        let update_data = UpdateClass {
            name: Some(existing_class.name),
            username: Some(existing_class.username),
            code: Some(existing_class.code),
            school_id: Some(existing_class.school_id),
            class_teacher_id: Some(existing_class.class_teacher_id),
            r#type: Some(existing_class.r#type),
            is_active: Some(existing_class.is_active),
            description: Some(existing_class.description),
            capacity: existing_class.capacity,
            subject: Some(existing_class.subject),
            grade_level: Some(existing_class.grade_level),
            tags: Some(existing_class.tags),
            image: existing_class.image,
            image_id: existing_class.image_id,
            background_images: existing_class.background_images,
        };

        let updated_class = self
            .repo
            .update_class(id, &update_data)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_class(updated_class))
    }

    /// Delete a class by id
    pub async fn delete_class(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_class(id).await.map_err(|e| e.message)
    }

    /// Count classes by school ID
    pub async fn count_classes_by_school_id(&self, school_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_school_id(school_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count classes by creator ID
    pub async fn count_classes_by_creator_id(&self, creator_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)
    }

    pub async fn create_many_classes(&self, classes: Vec<Class>) -> Result<Vec<Class>, String> {
        // Validate all classes first
        for class in &classes {
            is_valid_username(&class.username)?;
        }

        // Process classes: generate codes, set timestamps, etc.
        let mut processed_classes = Vec::with_capacity(classes.len());
        let now = Utc::now();

        for mut class in classes {
            // Generate class code if not provided
            if class.code.is_none() {
                class.code = Some(generate_code());
            }

            // Set timestamps
            class.created_at = now;
            class.updated_at = now;

            // Set default values for optional fields
            if !class.is_active {
                class.is_active = true;
            }

            // Generate ID
            class.id = Some(ObjectId::new());

            processed_classes.push(class);
        }

        // Create classes using repository
        let created_classes = self
            .repo
            .create_many_classes(processed_classes)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_classes(created_classes))
    }

    /// Create multiple classes with comprehensive validation
    pub async fn create_many_classes_with_validation(
        &self,
        classes: Vec<Class>,
    ) -> Result<Vec<Class>, String> {
        // Validate all classes first
        for class in &classes {
            is_valid_username(&class.username)?;
        }

        // Process classes: generate codes, set timestamps, etc.
        let mut processed_classes = Vec::with_capacity(classes.len());
        let now = Utc::now();

        for mut class in classes {
            // Generate class code if not provided
            if class.code.is_none() {
                class.code = Some(generate_code());
            }

            // Set timestamps
            class.created_at = now;
            class.updated_at = now;

            // Set default values for optional fields
            if !class.is_active {
                class.is_active = true;
            }

            // Generate ID
            class.id = Some(ObjectId::new());

            processed_classes.push(class);
        }

        // Create classes using repository with validation
        let created_classes = self
            .repo
            .create_many_classes_with_validation(processed_classes)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_classes(created_classes))
    }

    /// Create multiple classes for a specific school
    pub async fn create_many_classes_for_school(
        &self,
        school_id: &IdType,
        classes: Vec<Class>,
    ) -> Result<Vec<Class>, String> {
        // Validate all classes first
        for class in &classes {
            is_valid_username(&class.username)?;
        }

        // Process classes: generate codes, set timestamps, etc.
        let mut processed_classes = Vec::with_capacity(classes.len());
        let now = Utc::now();

        for mut class in classes {
            // Generate class code if not provided
            if class.code.is_none() {
                class.code = Some(generate_code());
            }

            // Set timestamps
            class.created_at = now;
            class.updated_at = now;

            // Set default values for optional fields
            if !class.is_active {
                class.is_active = true;
            }

            // Generate ID
            class.id = Some(ObjectId::new());

            processed_classes.push(class);
        }

        // Create classes for specific school
        let created_classes = self
            .repo
            .create_many_classes_for_school(school_id, processed_classes)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_classes(created_classes))
    }

    /// Bulk update multiple classes
    pub async fn update_many_classes(
        &self,
        updates: Vec<(IdType, UpdateClass)>,
    ) -> Result<Vec<Class>, String> {
        // Validate all updates first
        for (_, update) in &updates {
            if let Some(ref username) = update.username {
                is_valid_username(username)?;
            }
        }

        // Check uniqueness for usernames and codes that are being changed
        for (id, update) in &updates {
            if let Some(ref username) = update.username {
                // Get existing class to check if username is changing
                if let Ok(Some(existing_class)) = self.repo.find_by_id(id).await {
                    if existing_class.username != *username {
                        if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                            return Err(format!("Class username already exists: {}", username));
                        }
                    }
                }
            }

            if let Some(ref code) = update.code {
                // Get existing class to check if code is changing
                if let Ok(Some(existing_class)) = self.repo.find_by_id(id).await {
                    let existing_code = existing_class.code.as_ref();
                    let new_code = code.as_ref();

                    if existing_code != new_code {
                        if let Ok(Some(_)) = self
                            .repo
                            .find_by_code(new_code.unwrap_or(&"".to_string()))
                            .await
                        {
                            return Err("Class code already exists".to_string());
                        }
                    }
                }
            }
        }

        // Perform bulk update
        let updated_classes = self
            .repo
            .update_many_classes(updates)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_classes(updated_classes))
    }

    pub fn prepare_classes_for_bulk_creation(
        &self,
        classes: Vec<Class>,
        school_id: Option<ObjectId>,
        creator_id: Option<ObjectId>,
    ) -> Result<Vec<Class>, String> {
        let prepared_classes: Vec<Class> = classes
            .into_iter()
            .map(|mut class| {
                if let Some(sid) = school_id {
                    class.school_id = Some(sid);
                }
                if let Some(cid) = creator_id {
                    class.creator_id = Some(cid);
                }
                class
            })
            .collect();

        Ok(prepared_classes)
    }

    pub async fn get_many_classes_by_ids(&self, ids: Vec<ObjectId>) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .find_many_by_ids(ids)
            .await
            .map_err(|e| e.message.clone())?;

        Ok(sanitize_classes(classes))
    }
}
