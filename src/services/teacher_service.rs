use crate::{
    config::state::AppState,
    domain::{
        common_details::Gender,
        teacher::{
            BulkTeacherIds, BulkTeacherTags, BulkUpdateTeacherActive, PaginatedTeachers,
            PrepareTeacherRequest, Teacher, TeacherType, TeacherWithRelations, UpdateTeacher,
        },
    },
    helpers::object_id_helpers::parse_object_id,
    models::id_model::IdType,
    repositories::teacher_repo::TeacherRepo,
    services::{cloudinary_service::CloudinaryService, event_service::EventService},
    utils::{email::is_valid_email, names::is_valid_name},
};
use actix_web::web;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct TeacherService<'a> {
    repo: &'a TeacherRepo,
}

impl<'a> TeacherService<'a> {
    pub fn new(repo: &'a TeacherRepo) -> Self {
        Self { repo }
    }

    // ------------------------------------------------------------------
    // ✅ CRUD OPERATIONS
    // ------------------------------------------------------------------

    /// Get all teachers
    pub async fn get_all_teachers(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<PaginatedTeachers, String> {
        let teachers = self
            .repo
            .get_all_teachers(filter, limit, skip, None)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Get all teachers with relations
    pub async fn get_all_teachers_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<TeacherWithRelations>, String> {
        self.repo
            .get_all_with_relations(filter, limit, skip)
            .await
            .map_err(|e| e.message)
    }

    /// Get active teachers
    pub async fn get_active_teachers(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Teacher>, String> {
        let teachers = self
            .repo
            .get_active_teachers(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Create a new teacher
    pub async fn create_teacher(&self, mut new_teacher: Teacher) -> Result<Teacher, String> {
        // Validate name
        if let Err(e) = is_valid_name(&new_teacher.name) {
            return Err(format!("Invalid teacher name: {}", e));
        }

        // Validate email
        if let Err(e) = is_valid_email(&new_teacher.email) {
            return Err(format!("Invalid email: {}", e));
        }

        // Check if email already exists
        if let Ok(Some(_)) = self.repo.find_by_email(&new_teacher.email).await {
            return Err("Teacher email already exists".to_string());
        }

        // Check if user_id already exists (if provided)
        if let Some(user_id) = &new_teacher.user_id {
            if let Ok(Some(_)) = self
                .repo
                .find_by_user_id(&IdType::from_object_id(*user_id))
                .await
            {
                return Err("User ID already associated with another teacher".to_string());
            }
        }

        if let Some(image_data) = new_teacher.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data).await?;
            new_teacher.image_id = Some(cloud_res.public_id);
            new_teacher.image = Some(cloud_res.secure_url);
        }

        // Set timestamps
        let now = Utc::now();
        new_teacher.created_at = now;
        new_teacher.updated_at = now;

        // Set default values for optional fields
        if !new_teacher.is_active {
            new_teacher.is_active = true;
        }

        // Ensure tags is initialized
        if new_teacher.tags.is_empty() {
            new_teacher.tags = Vec::new();
        }

        if new_teacher
            .subject_ids
            .as_ref()
            .is_none_or(|v| v.is_empty())
        {
            new_teacher.subject_ids = None
        }

        if new_teacher.class_ids.as_ref().is_none_or(|v| v.is_empty()) {
            new_teacher.class_ids = None
        }

        // Set default type if not provided
        if matches!(new_teacher.r#type, TeacherType::Regular) {
            new_teacher.r#type = TeacherType::Regular;
        }

        // Generate ID
        let teacher_id = ObjectId::new();
        new_teacher.id = Some(teacher_id);

        // Save teacher in database
        let inserted_teacher = self
            .repo
            .insert_teacher(&new_teacher)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted_teacher)
    }

    // ------------------------------------------------------------------
    // ✅ CREATE WITH EVENTS
    // ------------------------------------------------------------------

    /// Get teacher by ID
    pub async fn get_teacher_by_id(&self, id: &IdType) -> Result<Teacher, String> {
        let teacher = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Teacher not found".to_string())?;

        Ok(teacher)
    }

    /// Get teacher by ID with relations
    pub async fn get_teacher_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<TeacherWithRelations, String> {
        let teacher = self
            .repo
            .find_by_id_with_relations(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Teacher not found".to_string())?;

        Ok(teacher)
    }

    /// Get teacher by user ID
    pub async fn get_teacher_by_user_id(&self, user_id: &IdType) -> Result<Teacher, String> {
        let teacher = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Teacher not found for this user".to_string())?;

        Ok(teacher)
    }

    /// Get teacher by email
    pub async fn get_teacher_by_email(&self, email: &str) -> Result<Teacher, String> {
        let teacher = self
            .repo
            .find_by_email(email)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Teacher not found".to_string())?;

        Ok(teacher)
    }

    /// Get teachers by school ID
    pub async fn get_teachers_by_school_id(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<Teacher>, String> {
        let teachers = self
            .repo
            .find_by_school_id(school_id)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Get teachers by class ID
    pub async fn get_teachers_by_class_id(
        &self,
        class_id: &IdType,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Teacher>, String> {
        let teachers = self
            .repo
            .find_by_class_id(class_id, filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Get teachers by subject ID
    pub async fn get_teachers_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<Teacher>, String> {
        let teachers = self
            .repo
            .find_by_subject_id(subject_id)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Get teachers by creator ID
    pub async fn get_teachers_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<Teacher>, String> {
        let teachers = self
            .repo
            .find_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Get teachers by type
    pub async fn get_teachers_by_type(
        &self,
        teacher_type: TeacherType,
    ) -> Result<Vec<Teacher>, String> {
        let teachers = self
            .repo
            .find_by_type(teacher_type)
            .await
            .map_err(|e| e.message)?;
        Ok(teachers)
    }

    /// Update a teacher
    pub async fn update_teacher(
        &self,
        id: &IdType,
        mut updated_data: UpdateTeacher,
    ) -> Result<Teacher, String> {
        // Validate name if provided
        if let Some(ref name) = updated_data.name {
            if let Err(e) = is_valid_name(name) {
                return Err(format!("Invalid teacher name: {}", e));
            }
        }

        // Validate email if provided
        if let Some(ref email) = updated_data.email {
            if let Err(e) = is_valid_email(email) {
                return Err(format!("Invalid email: {}", e));
            }
        }

        let existing_teacher = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Teacher not found".to_string())?;

        // ☁️ Replace profile image
        if let Some(new_image_data) = updated_data.image.clone() {
            if Some(new_image_data.clone()) != existing_teacher.image {
                if let Some(old_image_id) = existing_teacher.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data).await?;
                updated_data.image_id = Some(cloud_res.public_id);
                updated_data.image = Some(cloud_res.secure_url);
            }
        }

        // Check email uniqueness if provided and changed
        if let Some(ref email) = updated_data.email {
            if existing_teacher.email != *email {
                if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                    return Err("Teacher email already exists".to_string());
                }
            }
        }

        // Update teacher using repository method
        let updated_teacher = self
            .repo
            .update_teacher(id, &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teacher)
    }

    /// Delete a teacher by id
    pub async fn delete_teacher(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_teacher(id).await.map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ DELETE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn delete_teacher_with_events(
        &self,
        id: &IdType,
        state: &web::Data<AppState>,
    ) -> Result<(), String> {
        // Get teacher before deletion for broadcasting
        let teacher = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Teacher not found".to_string())?;

        self.delete_teacher(id).await?;

        // 🔔 Broadcast teacher deletion event
        if let Some(teacher_id) = &teacher.id {
            Self::broadcast_teacher_deletion(state, teacher_id, &teacher).await;
        }

        Ok(())
    }

    /// Count teachers by school ID
    pub async fn count_teachers_by_school_id(
        &self,
        school_id: &IdType,
        gender: Option<Gender>,
        teacher_type: Option<TeacherType>,
    ) -> Result<u64, String> {
        self.repo
            .count_by_school_id(school_id, gender, teacher_type)
            .await
            .map_err(|e| e.message)
    }

    /// Count teachers by class ID
    pub async fn count_teachers_by_class_id(&self, class_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_class_id(class_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count teachers by subject ID
    pub async fn count_teachers_by_subject_id(&self, subject_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_subject_id(subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count teachers by creator ID
    pub async fn count_teachers_by_creator_id(&self, creator_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count teachers by type
    pub async fn count_teachers_by_type(&self, teacher_type: TeacherType) -> Result<u64, String> {
        self.repo
            .count_by_type(teacher_type)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ BULK OPERATIONS WITH EVENTS
    // ------------------------------------------------------------------

    /// Create multiple teachers
    pub async fn create_many_teachers(
        &self,
        teachers: Vec<Teacher>,
    ) -> Result<Vec<Teacher>, String> {
        // Validate all teachers first
        for teacher in &teachers {
            if let Err(e) = is_valid_name(&teacher.name) {
                return Err(format!("Invalid teacher name '{}': {}", teacher.name, e));
            }

            if let Err(e) = is_valid_email(&teacher.email) {
                return Err(format!("Invalid email '{}': {}", teacher.email, e));
            }
        }

        // Process teachers: set timestamps, etc.
        let mut processed_teachers = Vec::with_capacity(teachers.len());
        let now = Utc::now();

        for mut teacher in teachers {
            // Set timestamps
            teacher.created_at = now;
            teacher.updated_at = now;

            // Set default values for optional fields
            if !teacher.is_active {
                teacher.is_active = true;
            }

            // Ensure tags is initialized
            if teacher.tags.is_empty() {
                teacher.tags = Vec::new();
            }

            // Set default type if not provided
            if matches!(teacher.r#type, TeacherType::Regular) {
                teacher.r#type = TeacherType::Regular;
            }

            // Generate ID
            teacher.id = Some(ObjectId::new());

            processed_teachers.push(teacher);
        }

        // Create teachers using repository
        let created_teachers = self
            .repo
            .create_many_teachers(processed_teachers)
            .await
            .map_err(|e| e.message)?;

        Ok(created_teachers)
    }

    /// Prepare teachers for bulk creation
    pub async fn prepare_teachers(
        &self,
        request: &PrepareTeacherRequest,
    ) -> Result<Vec<Teacher>, String> {
        self.repo
            .prepare_teachers(request)
            .await
            .map_err(|e| e.message)
    }

    /// Bulk update multiple teachers
    pub async fn update_many_teachers(
        &self,
        updates: Vec<(IdType, UpdateTeacher)>,
    ) -> Result<Vec<Teacher>, String> {
        // Validate all updates first
        for (_, update) in &updates {
            if let Some(ref name) = update.name {
                if let Err(e) = is_valid_name(name) {
                    return Err(format!("Invalid teacher name '{}': {}", name, e));
                }
            }

            if let Some(ref email) = update.email {
                if let Err(e) = is_valid_email(email) {
                    return Err(format!("Invalid email '{}': {}", email, e));
                }
            }
        }

        // Check uniqueness for emails that are being changed
        for (id, update) in &updates {
            if let Some(ref email) = update.email {
                // Get existing teacher to check if email is changing
                if let Ok(Some(existing_teacher)) = self.repo.find_by_id(id).await {
                    if existing_teacher.email != *email {
                        if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                            return Err(format!("Teacher email already exists: {}", email));
                        }
                    }
                }
            }
        }

        // Perform bulk update
        let updated_teachers = self
            .repo
            .update_many_teachers(updates)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teachers)
    }

    /// Bulk update active status for multiple teachers
    pub async fn bulk_update_active(
        &self,
        request: &BulkUpdateTeacherActive,
    ) -> Result<Vec<Teacher>, String> {
        let updated_teachers = self
            .repo
            .bulk_update_active(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teachers)
    }

    /// Bulk add tags to multiple teachers
    pub async fn bulk_add_tags(&self, request: &BulkTeacherTags) -> Result<Vec<Teacher>, String> {
        let updated_teachers = self
            .repo
            .bulk_add_tags(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teachers)
    }

    /// Bulk remove tags from multiple teachers
    pub async fn bulk_remove_tags(
        &self,
        request: &BulkTeacherTags,
    ) -> Result<Vec<Teacher>, String> {
        let updated_teachers = self
            .repo
            .bulk_remove_tags(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teachers)
    }

    /// Delete multiple teachers
    pub async fn delete_many_teachers(&self, request: &BulkTeacherIds) -> Result<u64, String> {
        self.repo
            .delete_many_teachers(request)
            .await
            .map_err(|e| e.message)
    }

    /// Broadcast teacher deletion event
    async fn broadcast_teacher_deletion(
        state: &web::Data<AppState>,
        teacher_id: &ObjectId,
        teacher: &Teacher,
    ) {
        let state_clone = state.clone();
        let teacher_id_clone = *teacher_id;
        let teacher_clone = teacher.clone();

        actix_rt::spawn(async move {
            EventService::broadcast_deleted(
                &state_clone,
                "teacher",
                &teacher_id_clone.to_hex(),
                &teacher_clone,
            )
            .await;
        });
    }

    // ------------------------------------------------------------------
    // 🔧 UTILITY METHODS
    // ------------------------------------------------------------------

    /// Check if a user is a teacher of a specific school
    pub async fn is_user_teacher_of_school(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<bool, String> {
        let teacher = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message)?;

        if let Some(teacher) = teacher {
            // Compare school IDs
            if let Some(teacher_school_id) = teacher.school_id {
                let target_school_id = parse_object_id(school_id)?;
                return Ok(teacher_school_id == target_school_id);
            }
        }

        Ok(false)
    }

    /// Get class teachers with details
    pub async fn get_class_teachers(
        &self,
        class_id: &IdType,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<TeacherWithRelations>, String> {
        let teachers = self
            .repo
            .get_all_with_relations(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;

        let class_obj_id = parse_object_id(class_id)?;
        let class_teachers: Vec<TeacherWithRelations> = teachers
            .into_iter()
            .filter(|teacher_with_rel| {
                teacher_with_rel
                    .teacher
                    .class_ids
                    .as_ref()
                    .map(|ids| ids.contains(&class_obj_id))
                    .unwrap_or(false)
            })
            .collect();

        Ok(class_teachers)
    }

    /// Get subject teachers with details
    pub async fn get_subject_teachers(
        &self,
        subject_id: &IdType,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<TeacherWithRelations>, String> {
        let teachers = self
            .repo
            .get_all_with_relations(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;

        let subject_obj_id = parse_object_id(subject_id)?;
        let subject_teachers: Vec<TeacherWithRelations> = teachers
            .into_iter()
            .filter(|teacher_with_rel| {
                teacher_with_rel
                    .teacher
                    .subject_ids
                    .as_ref()
                    .map(|ids| ids.contains(&subject_obj_id))
                    .unwrap_or(false)
            })
            .collect();

        Ok(subject_teachers)
    }

    /// Update teacher active status
    pub async fn update_teacher_active_status(
        &self,
        teacher_id: &IdType,
        is_active: bool,
    ) -> Result<Teacher, String> {
        let update_data = UpdateTeacher {
            is_active: Some(is_active),
            ..Default::default()
        };

        self.update_teacher(teacher_id, update_data).await
    }

    /// Activate a teacher
    pub async fn activate_teacher(&self, teacher_id: &IdType) -> Result<Teacher, String> {
        self.update_teacher_active_status(teacher_id, true).await
    }

    /// Deactivate a teacher
    pub async fn deactivate_teacher(&self, teacher_id: &IdType) -> Result<Teacher, String> {
        self.update_teacher_active_status(teacher_id, false).await
    }

    /// Add classes to teacher
    pub async fn add_classes_to_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, String> {
        let updated_teacher = self
            .repo
            .add_classes_to_teacher(teacher_id, class_ids)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teacher)
    }

    /// Add subjects to teacher
    pub async fn add_subjects_to_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, String> {
        let updated_teacher = self
            .repo
            .add_subjects_to_teacher(teacher_id, subject_ids)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teacher)
    }

    /// Remove classes from teacher
    pub async fn remove_classes_from_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, String> {
        let updated_teacher = self
            .repo
            .remove_classes_from_teacher(teacher_id, class_ids)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teacher)
    }

    /// Remove subjects from teacher
    pub async fn remove_subjects_from_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, String> {
        let updated_teacher = self
            .repo
            .remove_subjects_from_teacher(teacher_id, subject_ids)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_teacher)
    }

    /// Get teacher statistics for a school
    pub async fn get_school_teacher_statistics(
        &self,
        school_id: &IdType,
    ) -> Result<std::collections::HashMap<TeacherType, u64>, String> {
        let teachers = self.get_teachers_by_school_id(school_id).await?;

        let mut stats = std::collections::HashMap::new();
        for teacher in teachers {
            *stats.entry(teacher.r#type).or_insert(0) += 1;
        }

        Ok(stats)
    }
}
