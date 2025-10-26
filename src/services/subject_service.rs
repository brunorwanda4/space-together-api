use crate::{
    domain::{
        subject::{Subject, SubjectWithRelations, UpdateSubject},
        subjects::subject_category::SubjectCategory,
    },
    models::id_model::IdType,
    repositories::subject_repo::SubjectRepo,
    utils::{
        code::generate_code,
        names::is_valid_username,
        subject_utils::{sanitize_subject, sanitize_subjects},
    },
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SubjectService<'a> {
    repo: &'a SubjectRepo,
}

impl<'a> SubjectService<'a> {
    pub fn new(repo: &'a SubjectRepo) -> Self {
        Self { repo }
    }

    /// Get all subjects
    pub async fn get_all_subjects(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .get_all_subjects(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    /// Get all subjects with relations
    pub async fn get_all_subjects_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<SubjectWithRelations>, String> {
        self.repo
            .get_all_with_relations(filter, limit, skip)
            .await
            .map_err(|e| e.message)
    }

    /// Get active subjects
    pub async fn get_active_subjects(&self) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .get_active_subjects()
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    /// Create a new subject
    pub async fn create_subject(&self, mut new_subject: Subject) -> Result<Subject, String> {
        // ✅ Use the validation function
        self.validate_subject_data(&new_subject)?;

        // Check if code already exists
        if let Some(subject_code) = &new_subject.code {
            if let Ok(Some(_)) = self.repo.find_by_code(subject_code).await {
                return Err("Subject code already exists".to_string());
            }
        }

        // Check if username already exists
        if let Ok(Some(_)) = self.repo.find_by_username(&new_subject.username).await {
            return Err("Subject username already exists".to_string());
        }

        // Generate subject code if not provided
        if new_subject.code.is_none() {
            new_subject.code = Some(generate_code());
        }

        // Set timestamps
        let now = Utc::now();
        new_subject.created_at = now;
        new_subject.updated_at = now;

        // Set default values for optional fields
        if !new_subject.is_active {
            new_subject.is_active = true;
        }

        let subject_id = ObjectId::new();
        new_subject.id = Some(subject_id);

        // Save subject in database
        let inserted_subject = self
            .repo
            .insert_subject(&new_subject)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subject(inserted_subject))
    }

    /// Get subject by ID
    pub async fn get_subject_by_id(&self, id: &IdType) -> Result<Subject, String> {
        let subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        Ok(sanitize_subject(subject))
    }

    /// Get subject by ID with relations
    pub async fn get_subject_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<SubjectWithRelations, String> {
        let subject = self
            .repo
            .find_by_id_with_relations(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        Ok(subject)
    }

    /// Get subject by username
    pub async fn get_subject_by_username(&self, username: &str) -> Result<Subject, String> {
        let subject = self
            .repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        Ok(sanitize_subject(subject))
    }

    /// Get subject by username with relations
    pub async fn get_subject_by_username_with_relations(
        &self,
        username: &str,
    ) -> Result<SubjectWithRelations, String> {
        let subject = self
            .repo
            .find_by_username_with_relations(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        Ok(subject)
    }

    /// Get subject by code
    pub async fn get_subject_by_code(&self, code: &str) -> Result<Subject, String> {
        let subject = self
            .repo
            .find_by_code(code)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        Ok(sanitize_subject(subject))
    }

    /// Get subject by code with relations
    pub async fn get_subject_by_code_with_relations(
        &self,
        code: &str,
    ) -> Result<SubjectWithRelations, String> {
        let subject = self
            .repo
            .find_by_code_with_relations(code)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        Ok(subject)
    }

    /// Get subjects by class ID
    pub async fn get_subjects_by_class_id(
        &self,
        class_id: &IdType,
    ) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .find_by_class_id(class_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    /// Get subjects by creator ID
    pub async fn get_subjects_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .find_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    pub async fn get_subjects_by_class_teacher_id_with_relations(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<SubjectWithRelations>, String> {
        let subjects = self
            .repo
            .find_by_class_teacher_id_with_relations(teacher_id)
            .await
            .map_err(|e| e.message)?;
        Ok(subjects)
    }

    /// Get subjects by class teacher ID
    pub async fn get_subjects_by_class_teacher_id(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .find_by_class_teacher_id(teacher_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    /// Get subjects by main subject ID
    pub async fn get_subjects_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .find_by_main_subject_id(main_subject_id)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    /// Get subjects by subject type
    pub async fn get_subjects_by_subject_type(
        &self,
        subject_type: &SubjectCategory,
    ) -> Result<Vec<Subject>, String> {
        let subjects = self
            .repo
            .find_by_subject_type(subject_type)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_subjects(subjects))
    }

    /// Update a subject
    pub async fn update_subject(
        &self,
        id: &IdType,
        updated_data: UpdateSubject,
    ) -> Result<Subject, String> {
        let existing_subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        // Create a temporary subject for validation
        let mut temp_subject = existing_subject.clone();

        // Apply updates to temporary subject for validation
        if let Some(username) = updated_data.username.clone() {
            temp_subject.username = username;
        }
        if let Some(name) = updated_data.name.clone() {
            temp_subject.name = name;
        }
        if let Some(subject_type) = updated_data.subject_type.clone() {
            temp_subject.subject_type = subject_type;
        }
        if let Some(description) = updated_data.description.clone() {
            temp_subject.description = description;
        }

        // ✅ Use the validation function
        self.validate_subject_data(&temp_subject)?;

        // Check username uniqueness if provided and changed
        if let Some(ref username) = updated_data.username {
            if existing_subject.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("Subject username already exists".to_string());
                }
            }
        }

        // Check code uniqueness if provided and changed
        if let Some(ref code) = updated_data.code {
            if existing_subject.code.as_ref() != code.as_ref() {
                if let Ok(Some(_)) = self
                    .repo
                    .find_by_code(code.as_ref().unwrap_or(&"".to_string()))
                    .await
                {
                    return Err("Subject code already exists".to_string());
                }
            }
        }

        // Update subject using repository method
        let updated_subject = self
            .repo
            .update_subject(id, &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subject(updated_subject))
    }

    /// Alternative approach: update subject by merging with existing data
    pub async fn update_subject_merged(
        &self,
        id: &IdType,
        updated_data: UpdateSubject,
    ) -> Result<Subject, String> {
        let existing_subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        // Create a complete Subject object by merging existing data with updates
        let mut merged_subject = existing_subject.clone();

        // Update only the fields that are provided in updated_data
        if let Some(username) = updated_data.username {
            merged_subject.username = username;
        }
        if let Some(name) = updated_data.name {
            merged_subject.name = name;
        }
        if let Some(class_id) = updated_data.class_id {
            merged_subject.class_id = class_id;
        }
        if let Some(class_teacher_id) = updated_data.class_teacher_id {
            merged_subject.class_teacher_id = class_teacher_id;
        }
        if let Some(main_subject_id) = updated_data.main_subject_id {
            merged_subject.main_subject_id = main_subject_id;
        }
        if let Some(subject_type) = updated_data.subject_type {
            merged_subject.subject_type = subject_type;
        }
        if let Some(is_active) = updated_data.is_active {
            merged_subject.is_active = is_active;
        }
        if let Some(description) = updated_data.description {
            merged_subject.description = description;
        }
        if let Some(code) = updated_data.code {
            merged_subject.code = code;
        }
        if let Some(tags) = updated_data.tags {
            merged_subject.tags = tags;
        }

        // ✅ Use the validation function
        self.validate_subject_data(&merged_subject)?;

        // Check username uniqueness if provided and changed
        if !merged_subject.username.is_empty()
            && merged_subject.username != existing_subject.username
        {
            if let Ok(Some(_)) = self.repo.find_by_username(&merged_subject.username).await {
                return Err("Subject username already exists".to_string());
            }
        }

        // Check code uniqueness if provided and changed
        if let Some(ref code) = &merged_subject.code {
            if existing_subject.code.as_ref() != Some(code) {
                if let Ok(Some(_)) = self.repo.find_by_code(code).await {
                    return Err("Subject code already exists".to_string());
                }
            }
        }

        merged_subject.updated_at = Utc::now();

        // For merged update, we need to use the repository's update method
        // Since we don't have a method that takes a full Subject object, we'll convert to UpdateSubject
        let update_data = UpdateSubject {
            name: Some(merged_subject.name),
            username: Some(merged_subject.username),
            class_id: Some(merged_subject.class_id),
            class_teacher_id: Some(merged_subject.class_teacher_id),
            main_subject_id: Some(merged_subject.main_subject_id),
            subject_type: Some(merged_subject.subject_type),
            is_active: Some(merged_subject.is_active),
            description: Some(merged_subject.description),
            code: Some(merged_subject.code),
            tags: Some(merged_subject.tags),
        };

        let updated_subject = self
            .repo
            .update_subject(id, &update_data)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subject(updated_subject))
    }

    /// Delete a subject by id
    pub async fn delete_subject(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_subject(id).await.map_err(|e| e.message)
    }

    /// Count subjects by class ID
    pub async fn count_subjects_by_class_id(&self, class_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_class_id(class_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count subjects by creator ID
    pub async fn count_subjects_by_creator_id(&self, creator_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count subjects by class teacher ID
    pub async fn count_subjects_by_class_teacher_id(
        &self,
        teacher_id: &IdType,
    ) -> Result<u64, String> {
        self.repo
            .count_by_class_teacher_id(teacher_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count subjects by main subject ID
    pub async fn count_subjects_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<u64, String> {
        self.repo
            .count_by_main_subject_id(main_subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Validate subject data before creation or update
    pub fn validate_subject_data(&self, subject: &Subject) -> Result<(), String> {
        // Validate username and name using your existing utility functions
        is_valid_username(&subject.username)?;

        // Additional subject-specific validations
        if subject.name.trim().is_empty() {
            return Err("Subject name cannot be empty".to_string());
        }

        if subject.username.trim().is_empty() {
            return Err("Subject username cannot be empty".to_string());
        }

        // Validate subject type
        match &subject.subject_type {
            SubjectCategory::Other(custom_type) if custom_type.trim().is_empty() => {
                return Err("Custom subject type cannot be empty".to_string());
            }
            _ => {}
        }

        // Validate code format if present
        if let Some(code) = &subject.code {
            if code.trim().is_empty() {
                return Err("Subject code cannot be empty if provided".to_string());
            }
            if code.len() > 20 {
                return Err("Subject code cannot be longer than 20 characters".to_string());
            }
        }

        // Validate tags
        for tag in &subject.tags {
            if tag.trim().is_empty() {
                return Err("Subject tags cannot be empty".to_string());
            }
            if tag.len() > 50 {
                return Err("Subject tags cannot be longer than 50 characters".to_string());
            }
        }

        Ok(())
    }

    /// Toggle subject active status
    pub async fn toggle_subject_status(&self, id: &IdType) -> Result<Subject, String> {
        let subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        let update_data = UpdateSubject {
            is_active: Some(!subject.is_active),
            ..Default::default()
        };

        self.update_subject(id, update_data).await
    }

    /// Add tags to a subject
    pub async fn add_subject_tags(
        &self,
        id: &IdType,
        new_tags: Vec<String>,
    ) -> Result<Subject, String> {
        // Validate new tags before adding
        for tag in &new_tags {
            if tag.trim().is_empty() {
                return Err("Subject tags cannot be empty".to_string());
            }
            if tag.len() > 50 {
                return Err("Subject tags cannot be longer than 50 characters".to_string());
            }
        }

        let subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        let mut updated_tags = subject.tags.clone();
        for tag in new_tags {
            if !updated_tags.contains(&tag) {
                updated_tags.push(tag);
            }
        }

        let update_data = UpdateSubject {
            tags: Some(updated_tags),
            ..Default::default()
        };

        self.update_subject(id, update_data).await
    }

    /// Remove tags from a subject
    pub async fn remove_subject_tags(
        &self,
        id: &IdType,
        tags_to_remove: Vec<String>,
    ) -> Result<Subject, String> {
        let subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Subject not found".to_string())?;

        let updated_tags: Vec<String> = subject
            .tags
            .into_iter()
            .filter(|tag| !tags_to_remove.contains(tag))
            .collect();

        let update_data = UpdateSubject {
            tags: Some(updated_tags),
            ..Default::default()
        };

        self.update_subject(id, update_data).await
    }

    pub async fn create_many_subjects(
        &self,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, String> {
        // Validate all subjects first
        for subject in &subjects {
            self.validate_subject_data(subject)?;
        }

        // Process subjects: generate codes, set timestamps, etc.
        let mut processed_subjects = Vec::with_capacity(subjects.len());
        let now = Utc::now();

        for mut subject in subjects {
            // Generate subject code if not provided
            if subject.code.is_none() {
                subject.code = Some(generate_code());
            }

            // Set timestamps
            subject.created_at = now;
            subject.updated_at = now;

            // Set default values for optional fields
            if !subject.is_active {
                subject.is_active = true;
            }

            // Generate ID
            subject.id = Some(ObjectId::new());

            processed_subjects.push(subject);
        }

        // Create subjects using repository
        let created_subjects = self
            .repo
            .create_many_subjects(processed_subjects)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subjects(created_subjects))
    }

    /// Create multiple subjects with comprehensive validation
    pub async fn create_many_subjects_with_validation(
        &self,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, String> {
        // Validate all subjects first
        for subject in &subjects {
            self.validate_subject_data(subject)?;
        }

        // Process subjects: generate codes, set timestamps, etc.
        let mut processed_subjects = Vec::with_capacity(subjects.len());
        let now = Utc::now();

        for mut subject in subjects {
            // Generate subject code if not provided
            if subject.code.is_none() {
                subject.code = Some(generate_code());
            }

            // Set timestamps
            subject.created_at = now;
            subject.updated_at = now;

            // Set default values for optional fields
            if !subject.is_active {
                subject.is_active = true;
            }

            // Generate ID
            subject.id = Some(ObjectId::new());

            processed_subjects.push(subject);
        }

        // Create subjects using repository with validation
        let created_subjects = self
            .repo
            .create_many_subjects_with_validation(processed_subjects)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subjects(created_subjects))
    }

    /// Create multiple subjects for a specific class
    pub async fn create_many_subjects_for_class(
        &self,
        class_id: &IdType,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, String> {
        // Validate all subjects first
        for subject in &subjects {
            self.validate_subject_data(subject)?;
        }

        // Process subjects: generate codes, set timestamps, etc.
        let mut processed_subjects = Vec::with_capacity(subjects.len());
        let now = Utc::now();

        for mut subject in subjects {
            // Generate subject code if not provided
            if subject.code.is_none() {
                subject.code = Some(generate_code());
            }

            // Set timestamps
            subject.created_at = now;
            subject.updated_at = now;

            // Set default values for optional fields
            if !subject.is_active {
                subject.is_active = true;
            }

            // Generate ID
            subject.id = Some(ObjectId::new());

            processed_subjects.push(subject);
        }

        // Create subjects for specific class
        let created_subjects = self
            .repo
            .create_many_subjects_for_class(class_id, processed_subjects)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subjects(created_subjects))
    }

    /// Create multiple subjects for a specific teacher
    pub async fn create_many_subjects_for_teacher(
        &self,
        teacher_id: &IdType,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, String> {
        // Validate all subjects first
        for subject in &subjects {
            self.validate_subject_data(subject)?;
        }

        // Process subjects: generate codes, set timestamps, etc.
        let mut processed_subjects = Vec::with_capacity(subjects.len());
        let now = Utc::now();

        for mut subject in subjects {
            // Generate subject code if not provided
            if subject.code.is_none() {
                subject.code = Some(generate_code());
            }

            // Set timestamps
            subject.created_at = now;
            subject.updated_at = now;

            // Set default values for optional fields
            if !subject.is_active {
                subject.is_active = true;
            }

            // Generate ID
            subject.id = Some(ObjectId::new());

            processed_subjects.push(subject);
        }

        // Create subjects for specific teacher
        let created_subjects = self
            .repo
            .create_many_subjects_for_teacher(teacher_id, processed_subjects)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subjects(created_subjects))
    }

    /// Create multiple subjects for a specific main subject
    pub async fn create_many_subjects_for_main_subject(
        &self,
        main_subject_id: &IdType,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, String> {
        // Validate all subjects first
        for subject in &subjects {
            self.validate_subject_data(subject)?;
        }

        // Process subjects: generate codes, set timestamps, etc.
        let mut processed_subjects = Vec::with_capacity(subjects.len());
        let now = Utc::now();

        for mut subject in subjects {
            // Generate subject code if not provided
            if subject.code.is_none() {
                subject.code = Some(generate_code());
            }

            // Set timestamps
            subject.created_at = now;
            subject.updated_at = now;

            // Set default values for optional fields
            if !subject.is_active {
                subject.is_active = true;
            }

            // Generate ID
            subject.id = Some(ObjectId::new());

            processed_subjects.push(subject);
        }

        // Create subjects for specific main subject
        let created_subjects = self
            .repo
            .create_many_subjects_for_main_subject(main_subject_id, processed_subjects)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subjects(created_subjects))
    }

    /// Bulk update multiple subjects
    pub async fn update_many_subjects(
        &self,
        updates: Vec<(IdType, UpdateSubject)>,
    ) -> Result<Vec<Subject>, String> {
        // Validate all updates first
        for (id, update) in &updates {
            // Create a temporary subject for validation by getting existing subject and applying updates
            if let Ok(Some(existing_subject)) = self.repo.find_by_id(id).await {
                let mut temp_subject = existing_subject.clone();

                // Apply updates to temporary subject for validation
                if let Some(ref username) = update.username {
                    temp_subject.username = username.clone();
                }
                if let Some(ref name) = update.name {
                    temp_subject.name = name.clone();
                }
                if let Some(ref subject_type) = update.subject_type {
                    temp_subject.subject_type = subject_type.clone();
                }

                self.validate_subject_data(&temp_subject)?;
            }
        }

        // Check uniqueness for usernames and codes that are being changed
        for (id, update) in &updates {
            if let Some(ref username) = update.username {
                // Get existing subject to check if username is changing
                if let Ok(Some(existing_subject)) = self.repo.find_by_id(id).await {
                    if existing_subject.username != *username {
                        if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                            return Err(format!("Subject username already exists: {}", username));
                        }
                    }
                }
            }

            if let Some(ref code) = update.code {
                // Get existing subject to check if code is changing
                if let Ok(Some(existing_subject)) = self.repo.find_by_id(id).await {
                    let existing_code = existing_subject.code.as_ref();
                    let new_code = code.as_ref();

                    if existing_code != new_code {
                        if let Ok(Some(_)) = self
                            .repo
                            .find_by_code(new_code.unwrap_or(&"".to_string()))
                            .await
                        {
                            return Err("Subject code already exists".to_string());
                        }
                    }
                }
            }
        }

        // Perform bulk update
        let updated_subjects = self
            .repo
            .update_many_subjects(updates)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_subjects(updated_subjects))
    }

    /// Bulk delete multiple subjects
    pub async fn delete_many_subjects(&self, ids: Vec<IdType>) -> Result<u64, String> {
        let deleted_count = self
            .repo
            .delete_many_subjects(ids)
            .await
            .map_err(|e| e.message)?;

        Ok(deleted_count)
    }

    /// Get subjects by multiple IDs
    pub async fn get_subjects_by_ids(&self, ids: Vec<IdType>) -> Result<Vec<Subject>, String> {
        let subjects = self.repo.find_by_ids(ids).await.map_err(|e| e.message)?;

        Ok(sanitize_subjects(subjects))
    }

    /// Get subjects with relations by multiple IDs
    pub async fn get_subjects_by_ids_with_relations(
        &self,
        ids: Vec<IdType>,
    ) -> Result<Vec<SubjectWithRelations>, String> {
        let subjects = self
            .repo
            .find_by_ids_with_relations(ids)
            .await
            .map_err(|e| e.message)?;

        Ok(subjects)
    }

    /// Check if identifiers (usernames/codes) already exist in bulk
    pub async fn check_existing_identifiers(
        &self,
        usernames: &[String],
        codes: &[String],
    ) -> Result<(Vec<String>, Vec<String>), String> {
        let (existing_usernames, existing_codes) = self
            .repo
            .check_existing_identifiers(usernames, codes)
            .await
            .map_err(|e| e.message)?;

        Ok((existing_usernames, existing_codes))
    }

    /// Bulk update subjects by class ID
    pub async fn update_many_subjects_by_class_id(
        &self,
        class_id: &IdType,
        update: UpdateSubject,
    ) -> Result<u64, String> {
        // Validate the update data if it contains fields that need validation
        if let Some(ref username) = update.username {
            is_valid_username(username)?;
        }

        let updated_count = self
            .repo
            .update_many_by_class_id(class_id, &update)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_count)
    }

    /// Bulk activate/deactivate subjects
    pub async fn bulk_update_subjects_active_status(
        &self,
        ids: Vec<IdType>,
        is_active: bool,
    ) -> Result<u64, String> {
        let updated_count = self
            .repo
            .bulk_update_active_status(ids, is_active)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_count)
    }

    /// Bulk toggle subject status for multiple subjects
    pub async fn bulk_toggle_subjects_status(
        &self,
        ids: Vec<IdType>,
    ) -> Result<Vec<Subject>, String> {
        let mut updates = Vec::with_capacity(ids.len());

        // First, get current status of all subjects
        let subjects = self.get_subjects_by_ids(ids.clone()).await?;

        for subject in subjects {
            let update = UpdateSubject {
                is_active: Some(!subject.is_active),
                ..Default::default()
            };
            updates.push((IdType::from_object_id(subject.id.unwrap()), update));
        }

        // Perform bulk update
        self.update_many_subjects(updates).await
    }

    /// Bulk add tags to multiple subjects
    pub async fn bulk_add_subjects_tags(
        &self,
        ids: Vec<IdType>,
        new_tags: Vec<String>,
    ) -> Result<Vec<Subject>, String> {
        // Validate new tags before adding
        for tag in &new_tags {
            if tag.trim().is_empty() {
                return Err("Subject tags cannot be empty".to_string());
            }
            if tag.len() > 50 {
                return Err("Subject tags cannot be longer than 50 characters".to_string());
            }
        }

        let mut updates = Vec::with_capacity(ids.len());
        let subjects = self.get_subjects_by_ids(ids.clone()).await?;

        for subject in subjects {
            let mut updated_tags = subject.tags.clone();
            for tag in &new_tags {
                if !updated_tags.contains(tag) {
                    updated_tags.push(tag.clone());
                }
            }

            let update = UpdateSubject {
                tags: Some(updated_tags),
                ..Default::default()
            };
            updates.push((IdType::from_object_id(subject.id.unwrap()), update));
        }

        self.update_many_subjects(updates).await
    }

    /// Bulk remove tags from multiple subjects
    pub async fn bulk_remove_subjects_tags(
        &self,
        ids: Vec<IdType>,
        tags_to_remove: Vec<String>,
    ) -> Result<Vec<Subject>, String> {
        let mut updates = Vec::with_capacity(ids.len());
        let subjects = self.get_subjects_by_ids(ids.clone()).await?;

        for subject in subjects {
            let updated_tags: Vec<String> = subject
                .tags
                .into_iter()
                .filter(|tag| !tags_to_remove.contains(tag))
                .collect();

            let update = UpdateSubject {
                tags: Some(updated_tags),
                ..Default::default()
            };
            updates.push((IdType::from_object_id(subject.id.unwrap()), update));
        }

        self.update_many_subjects(updates).await
    }
}
