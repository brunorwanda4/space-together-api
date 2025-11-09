use crate::{
    domain::{
        class::{Class, ClassWithOthers, PaginatedClasses, UpdateClass},
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
    ) -> Result<PaginatedClasses, String> {
        let classes = self
            .repo
            .get_all_classes(filter, limit, skip, None)
            .await
            .map_err(|e| e.message)?;
        Ok(PaginatedClasses {
            classes: sanitize_classes(classes.classes),
            total: classes.total,
            total_pages: classes.total_pages,
            current_page: classes.current_page,
        })
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

            if class.image.is_some() {
                class.image = None;
                class.image_id = None;
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

    pub async fn get_many_classes_by_ids(&self, ids: Vec<ObjectId>) -> Result<Vec<Class>, String> {
        let classes = self
            .repo
            .find_many_by_ids(ids)
            .await
            .map_err(|e| e.message.clone())?;

        Ok(sanitize_classes(classes))
    }

    // ===========================
    // SUBCLASS MANAGEMENT METHODS
    // ===========================

    /// Add a subclass to a main class
    pub async fn add_subclass(
        &self,
        main_class_id: &IdType,
        subclass: Class,
    ) -> Result<Class, String> {
        // Validate subclass username
        is_valid_username(&subclass.username)?;

        // Check if subclass username already exists
        if let Ok(Some(_)) = self.repo.find_by_username(&subclass.username).await {
            return Err("Subclass username already exists".to_string());
        }

        // Process subclass data (images, code generation, etc.)
        let mut processed_subclass = subclass;

        // Handle image upload for subclass
        if let Some(image_data) = processed_subclass.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data).await?;
            processed_subclass.image_id = Some(cloud_res.public_id);
            processed_subclass.image = Some(cloud_res.secure_url);
        }

        // Handle background images for subclass
        if let Some(background_images_data) = processed_subclass.background_images.clone() {
            let mut uploaded_images = Vec::new();
            for bg in background_images_data {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url).await?;
                uploaded_images.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            processed_subclass.background_images = Some(uploaded_images);
        }

        // Generate class code if not provided
        if processed_subclass.code.is_none() {
            processed_subclass.code = Some(generate_code());
        }

        // Set timestamps
        let now = Utc::now();
        processed_subclass.created_at = now;
        processed_subclass.updated_at = now;

        // Set default values
        if !processed_subclass.is_active {
            processed_subclass.is_active = true;
        }

        // Add subclass using repository
        let inserted_subclass = self
            .repo
            .add_subclass(main_class_id, &processed_subclass)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_class(inserted_subclass))
    }

    /// Remove a subclass from its main class
    pub async fn remove_subclass(&self, subclass_id: &IdType) -> Result<(), String> {
        // Get subclass to check for images that need cleanup
        if let Ok(Some(subclass)) = self.repo.find_by_id(subclass_id).await {
            // Clean up images from cloud storage
            if let Some(image_id) = subclass.image_id {
                let _ = CloudinaryService::delete_from_cloudinary(&image_id).await;
            }

            if let Some(background_images) = subclass.background_images {
                for bg in background_images {
                    let _ = CloudinaryService::delete_from_cloudinary(&bg.id).await;
                }
            }
        }

        self.repo
            .remove_subclass(subclass_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get all subclasses of a main class
    pub async fn get_subclasses(&self, main_class_id: &IdType) -> Result<Vec<Class>, String> {
        let subclasses = self
            .repo
            .get_subclasses(main_class_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_classes(subclasses))
    }

    /// Get subclasses with full details (including school, teacher, etc.)
    pub async fn get_subclasses_with_others(
        &self,
        main_class_id: &IdType,
    ) -> Result<Vec<ClassWithOthers>, String> {
        self.repo
            .get_subclasses_with_others(main_class_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get the main class of a subclass
    pub async fn get_parent_class(&self, subclass_id: &IdType) -> Result<Option<Class>, String> {
        let parent_class = self
            .repo
            .get_parent_class(subclass_id)
            .await
            .map_err(|e| e.message)?;

        Ok(parent_class.map(sanitize_class))
    }

    /// Move a subclass to a different main class
    pub async fn move_subclass(
        &self,
        subclass_id: &IdType,
        new_main_class_id: &IdType,
    ) -> Result<Class, String> {
        let updated_subclass = self
            .repo
            .move_subclass(subclass_id, new_main_class_id)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_class(updated_subclass))
    }

    /// Check if a class is a main class with subclasses
    pub async fn is_main_class_with_subclasses(&self, class_id: &IdType) -> Result<bool, String> {
        self.repo
            .is_main_class_with_subclasses(class_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get all main classes (classes without parent_class_id and with MainClass level type)
    pub async fn get_main_classes(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<PaginatedClasses, String> {
        let main_classes = self
            .repo
            .get_main_classes(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(PaginatedClasses {
            total: main_classes.total,
            total_pages: main_classes.total_pages,
            current_page: main_classes.current_page,
            classes: sanitize_classes(main_classes.classes),
        })
    }

    /// Bulk add multiple subclasses to a main class
    pub async fn add_multiple_subclasses(
        &self,
        main_class_id: &IdType,
        subclasses: Vec<Class>,
    ) -> Result<Vec<Class>, String> {
        // Validate all subclasses first
        for subclass in &subclasses {
            is_valid_username(&subclass.username)?;

            // Check if subclass username already exists
            if let Ok(Some(_)) = self.repo.find_by_username(&subclass.username).await {
                return Err(format!(
                    "Subclass username already exists: {}",
                    subclass.username
                ));
            }
        }

        // Process subclasses: generate codes, set timestamps, etc.
        let mut processed_subclasses = Vec::with_capacity(subclasses.len());
        let now = Utc::now();

        for mut subclass in subclasses {
            // Generate class code if not provided
            if subclass.code.is_none() {
                subclass.code = Some(generate_code());
            }

            // Set timestamps
            subclass.created_at = now;
            subclass.updated_at = now;

            // Set default values for optional fields
            if !subclass.is_active {
                subclass.is_active = true;
            }

            // Clear images for bulk processing (they can be updated later)
            // if subclass.image.is_some() {
            //     subclass.image = None;
            //     subclass.image_id = None;
            // }
            // if subclass.background_images.is_some() {
            //     subclass.background_images = None;
            // }

            // Generate ID
            subclass.id = Some(ObjectId::new());

            processed_subclasses.push(subclass);
        }

        // Add subclasses using repository
        let inserted_subclasses = self
            .repo
            .add_multiple_subclasses(main_class_id, processed_subclasses)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_classes(inserted_subclasses))
    }

    /// Create a main class (convenience method)
    pub async fn create_main_class(&self, mut main_class: Class) -> Result<Class, String> {
        // Ensure level_type is set to MainClass
        main_class.level_type = Some(crate::domain::class::ClassLevelType::MainClass);
        main_class.parent_class_id = None;
        main_class.subclass_ids = Some(Vec::new());

        self.create_class(main_class).await
    }

    /// Check if a class can be deleted (no subclasses if it's a main class)
    pub async fn can_delete_class(&self, class_id: &IdType) -> Result<bool, String> {
        let class = self
            .repo
            .find_by_id(class_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Class not found".to_string())?;

        // If it's a main class, check if it has subclasses
        if class.level_type == Some(crate::domain::class::ClassLevelType::MainClass) {
            if let Some(subclass_ids) = class.subclass_ids {
                return Ok(subclass_ids.is_empty());
            }
        }

        Ok(true)
    }

    /// Get class hierarchy (main class with all its subclasses)
    pub async fn get_class_hierarchy(
        &self,
        main_class_id: &IdType,
    ) -> Result<(Class, Vec<Class>), String> {
        let main_class = self.get_class_by_id(main_class_id).await?;

        let subclasses = self.get_subclasses(main_class_id).await?;

        Ok((main_class, subclasses))
    }
}
