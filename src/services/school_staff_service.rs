use crate::{
    config::state::AppState,
    domain::school_staff::{
        BulkIdsRequest, BulkTagsRequest, BulkUpdateActiveStatusRequest, SchoolStaff,
        SchoolStaffType, SchoolStaffWithRelations, UpdateSchoolStaff,
    },
    helpers::object_id_helpers::parse_object_id,
    models::id_model::IdType,
    repositories::school_staff_repo::SchoolStaffRepo,
    services::event_service::EventService,
    utils::{email::is_valid_email, names::is_valid_name},
};
use actix_web::web;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SchoolStaffService<'a> {
    repo: &'a SchoolStaffRepo,
}

impl<'a> SchoolStaffService<'a> {
    pub fn new(repo: &'a SchoolStaffRepo) -> Self {
        Self { repo }
    }

    // ------------------------------------------------------------------
    // ✅ CRUD OPERATIONS
    // ------------------------------------------------------------------

    /// Get all school staff members
    pub async fn get_all_school_staff(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<SchoolStaff>, String> {
        let staff_members = self
            .repo
            .get_all_school_staff(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(staff_members)
    }

    /// Get all school staff members with relations
    pub async fn get_all_school_staff_with_relations(
        &self,
    ) -> Result<Vec<SchoolStaffWithRelations>, String> {
        self.repo
            .get_all_with_relations()
            .await
            .map_err(|e| e.message)
    }

    /// Get active school staff members
    pub async fn get_active_school_staff(&self) -> Result<Vec<SchoolStaff>, String> {
        let staff_members = self
            .repo
            .get_active_school_staff()
            .await
            .map_err(|e| e.message)?;
        Ok(staff_members)
    }

    /// Create a new school staff member
    pub async fn create_school_staff(
        &self,
        mut new_staff: SchoolStaff,
    ) -> Result<SchoolStaff, String> {
        // Validate name
        if let Err(e) = is_valid_name(&new_staff.name) {
            return Err(format!("Invalid staff name: {}", e));
        }

        // Validate email
        if let Err(e) = is_valid_email(&new_staff.email) {
            return Err(format!("Invalid email: {}", e));
        }

        // Check if email already exists
        if let Ok(Some(_)) = self.repo.find_by_email(&new_staff.email).await {
            return Err("Staff email already exists".to_string());
        }

        // Check if user_id already exists (if provided)
        if let Some(user_id) = &new_staff.user_id {
            if let Ok(Some(_)) = self
                .repo
                .find_by_user_id(&IdType::from_object_id(*user_id))
                .await
            {
                return Err("User ID already associated with another staff member".to_string());
            }
        }

        // Set timestamps
        let now = Utc::now();
        new_staff.created_at = now;
        new_staff.updated_at = now;

        // Set default values for optional fields
        if !new_staff.is_active {
            new_staff.is_active = true;
        }

        // Ensure tags is initialized
        if new_staff.tags.is_empty() {
            new_staff.tags = Vec::new();
        }

        // Generate ID
        let staff_id = ObjectId::new();
        new_staff.id = Some(staff_id);

        // Save staff member in database
        let inserted_staff = self
            .repo
            .insert_school_staff(&new_staff)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted_staff)
    }

    // ------------------------------------------------------------------
    // ✅ CREATE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn create_school_staff_with_events(
        &self,
        new_staff: SchoolStaff,
        state: &web::Data<AppState>,
    ) -> Result<SchoolStaff, String> {
        let staff = self.create_school_staff(new_staff).await?;

        // 🔔 Broadcast school staff creation event
        if let Some(id) = &staff.id {
            Self::broadcast_school_staff_update(state, id).await;
        }

        Ok(staff)
    }

    /// Get school staff by ID
    pub async fn get_school_staff_by_id(&self, id: &IdType) -> Result<SchoolStaff, String> {
        let staff = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found".to_string())?;

        Ok(staff)
    }

    /// Get school staff by ID with relations
    pub async fn get_school_staff_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<SchoolStaffWithRelations, String> {
        let staff = self
            .repo
            .find_by_id_with_relations(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found".to_string())?;

        Ok(staff)
    }

    /// Get school staff by user ID
    pub async fn get_school_staff_by_user_id(
        &self,
        user_id: &IdType,
    ) -> Result<SchoolStaff, String> {
        let staff = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found for this user".to_string())?;

        Ok(staff)
    }

    /// Get school staff by email
    pub async fn get_school_staff_by_email(&self, email: &str) -> Result<SchoolStaff, String> {
        let staff = self
            .repo
            .find_by_email(email)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found".to_string())?;

        Ok(staff)
    }

    /// Get school staff by school ID
    pub async fn get_school_staff_by_school_id(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<SchoolStaff>, String> {
        let staff_members = self
            .repo
            .find_by_school_id(school_id)
            .await
            .map_err(|e| e.message)?;
        Ok(staff_members)
    }

    /// Get school staff by creator ID
    pub async fn get_school_staff_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<SchoolStaff>, String> {
        let staff_members = self
            .repo
            .find_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)?;
        Ok(staff_members)
    }

    /// Get school staff by type
    pub async fn get_school_staff_by_type(
        &self,
        staff_type: SchoolStaffType,
    ) -> Result<Vec<SchoolStaff>, String> {
        let staff_members = self
            .repo
            .find_by_type(staff_type)
            .await
            .map_err(|e| e.message)?;
        Ok(staff_members)
    }

    /// Get school staff by school ID and type
    pub async fn get_school_staff_by_school_and_type(
        &self,
        school_id: &IdType,
        staff_type: SchoolStaffType,
    ) -> Result<Vec<SchoolStaff>, String> {
        let staff_members = self
            .repo
            .find_by_school_and_type(school_id, staff_type)
            .await
            .map_err(|e| e.message)?;
        Ok(staff_members)
    }

    /// Update a school staff member
    pub async fn update_school_staff(
        &self,
        id: &IdType,
        updated_data: UpdateSchoolStaff,
    ) -> Result<SchoolStaff, String> {
        // Validate name if provided
        if let Some(ref name) = updated_data.name {
            if let Err(e) = is_valid_name(name) {
                return Err(format!("Invalid staff name: {}", e));
            }
        }

        // Validate email if provided
        if let Some(ref email) = updated_data.email {
            if let Err(e) = is_valid_email(email) {
                return Err(format!("Invalid email: {}", e));
            }
        }

        let existing_staff = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found".to_string())?;

        // Check email uniqueness if provided and changed
        if let Some(ref email) = updated_data.email {
            if existing_staff.email != *email {
                if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                    return Err("Staff email already exists".to_string());
                }
            }
        }

        // Update staff using repository method
        let updated_staff = self
            .repo
            .update_school_staff(id, &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_staff)
    }

    // ------------------------------------------------------------------
    // ✅ UPDATE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn update_school_staff_with_events(
        &self,
        id: &IdType,
        updated_data: UpdateSchoolStaff,
        state: &web::Data<AppState>,
    ) -> Result<SchoolStaff, String> {
        let updated_staff = self.update_school_staff(id, updated_data).await?;

        // 🔔 Broadcast school staff update event
        if let Some(staff_id) = &updated_staff.id {
            Self::broadcast_school_staff_update(state, staff_id).await;
        }

        Ok(updated_staff)
    }

    /// Alternative approach: update school staff by merging with existing data
    pub async fn update_school_staff_merged(
        &self,
        id: &IdType,
        updated_data: UpdateSchoolStaff,
    ) -> Result<SchoolStaff, String> {
        // Validate name if provided
        if let Some(ref name) = updated_data.name {
            if let Err(e) = is_valid_name(name) {
                return Err(format!("Invalid staff name: {}", e));
            }
        }

        // Validate email if provided
        if let Some(ref email) = updated_data.email {
            if let Err(e) = is_valid_email(email) {
                return Err(format!("Invalid email: {}", e));
            }
        }

        let existing_staff = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found".to_string())?;

        // Check email uniqueness if provided and changed
        if let Some(ref email) = updated_data.email {
            if existing_staff.email != *email {
                if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                    return Err("Staff email already exists".to_string());
                }
            }
        }

        // Create a complete SchoolStaff object by merging existing data with updates
        let mut merged_staff = existing_staff.clone();

        // Update only the fields that are provided in updated_data
        if let Some(name) = updated_data.name {
            merged_staff.name = name;
        }
        if let Some(email) = updated_data.email {
            merged_staff.email = email;
        }
        if let Some(staff_type) = updated_data.r#type {
            merged_staff.r#type = staff_type;
        }
        if let Some(is_active) = updated_data.is_active {
            merged_staff.is_active = is_active;
        }
        if let Some(tags) = updated_data.tags {
            merged_staff.tags = tags;
        }

        merged_staff.updated_at = Utc::now();

        // Since we don't have a method that takes a full SchoolStaff object, we'll convert to UpdateSchoolStaff
        let update_data = UpdateSchoolStaff {
            name: Some(merged_staff.name),
            email: Some(merged_staff.email),
            r#type: Some(merged_staff.r#type),
            is_active: Some(merged_staff.is_active),
            tags: Some(merged_staff.tags),
        };

        let updated_staff = self
            .repo
            .update_school_staff(id, &update_data)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_staff)
    }

    /// Delete a school staff member by id
    pub async fn delete_school_staff(&self, id: &IdType) -> Result<(), String> {
        self.repo
            .delete_school_staff(id)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ DELETE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn delete_school_staff_with_events(
        &self,
        id: &IdType,
        state: &web::Data<AppState>,
    ) -> Result<(), String> {
        // Get staff before deletion for broadcasting
        let staff = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School staff not found".to_string())?;

        self.delete_school_staff(id).await?;

        // 🔔 Broadcast school staff deletion event
        if let Some(staff_id) = &staff.id {
            Self::broadcast_school_staff_deletion(state, staff_id, &staff).await;
        }

        Ok(())
    }

    /// Count school staff by school ID
    pub async fn count_staff_by_school_id(
        &self,
        school_id: &IdType,
        staff_type: Option<SchoolStaffType>,
        is_active: Option<bool>,
    ) -> Result<u64, String> {
        self.repo
            .count_by_school_id(school_id, staff_type, is_active)
            .await
            .map_err(|e| e.message)
    }

    /// Count school staff by creator ID
    pub async fn count_school_staff_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<u64, String> {
        self.repo
            .count_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count school staff by type
    pub async fn count_school_staff_by_type(
        &self,
        staff_type: SchoolStaffType,
    ) -> Result<u64, String> {
        self.repo
            .count_by_type(staff_type)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ BULK OPERATIONS WITH EVENTS
    // ------------------------------------------------------------------

    /// Create multiple school staff members
    pub async fn create_many_school_staff(
        &self,
        staff_members: Vec<SchoolStaff>,
    ) -> Result<Vec<SchoolStaff>, String> {
        // Validate all staff members first
        for staff in &staff_members {
            if let Err(e) = is_valid_name(&staff.name) {
                return Err(format!("Invalid staff name '{}': {}", staff.name, e));
            }

            if let Err(e) = is_valid_email(&staff.email) {
                return Err(format!("Invalid email '{}': {}", staff.email, e));
            }
        }

        // Process staff members: set timestamps, etc.
        let mut processed_staff = Vec::with_capacity(staff_members.len());
        let now = Utc::now();

        for mut staff in staff_members {
            // Set timestamps
            staff.created_at = now;
            staff.updated_at = now;

            // Set default values for optional fields
            if !staff.is_active {
                staff.is_active = true;
            }

            // Ensure tags is initialized
            if staff.tags.is_empty() {
                staff.tags = Vec::new();
            }

            // Generate ID
            staff.id = Some(ObjectId::new());

            processed_staff.push(staff);
        }

        // Create staff members using repository
        let created_staff = self
            .repo
            .create_many_school_staff(processed_staff)
            .await
            .map_err(|e| e.message)?;

        Ok(created_staff)
    }

    /// Create multiple school staff members with events
    pub async fn create_many_school_staff_with_events(
        &self,
        staff_members: Vec<SchoolStaff>,
        state: &web::Data<AppState>,
    ) -> Result<Vec<SchoolStaff>, String> {
        let created_staff = self.create_many_school_staff(staff_members).await?;

        // 🔔 Broadcast creation events for all created staff
        for staff in &created_staff {
            if let Some(id) = &staff.id {
                Self::broadcast_school_staff_update(state, id).await;
            }
        }

        Ok(created_staff)
    }

    /// Create multiple school staff members with comprehensive validation
    pub async fn create_many_school_staff_with_validation(
        &self,
        staff_members: Vec<SchoolStaff>,
    ) -> Result<Vec<SchoolStaff>, String> {
        // Validate all staff members first
        for staff in &staff_members {
            if let Err(e) = is_valid_name(&staff.name) {
                return Err(format!("Invalid staff name '{}': {}", staff.name, e));
            }

            if let Err(e) = is_valid_email(&staff.email) {
                return Err(format!("Invalid email '{}': {}", staff.email, e));
            }
        }

        // Process staff members: set timestamps, etc.
        let mut processed_staff = Vec::with_capacity(staff_members.len());
        let now = Utc::now();

        for mut staff in staff_members {
            // Set timestamps
            staff.created_at = now;
            staff.updated_at = now;

            // Set default values for optional fields
            if !staff.is_active {
                staff.is_active = true;
            }

            // Ensure tags is initialized
            if staff.tags.is_empty() {
                staff.tags = Vec::new();
            }

            // Generate ID
            staff.id = Some(ObjectId::new());

            processed_staff.push(staff);
        }

        // Create staff members using repository with validation
        let created_staff = self
            .repo
            .create_many_school_staff_with_validation(processed_staff)
            .await
            .map_err(|e| e.message)?;

        Ok(created_staff)
    }

    /// Bulk update multiple school staff members
    pub async fn update_many_school_staff(
        &self,
        updates: Vec<(IdType, UpdateSchoolStaff)>,
    ) -> Result<Vec<SchoolStaff>, String> {
        // Validate all updates first
        for (_, update) in &updates {
            if let Some(ref name) = update.name {
                if let Err(e) = is_valid_name(name) {
                    return Err(format!("Invalid staff name '{}': {}", name, e));
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
                // Get existing staff to check if email is changing
                if let Ok(Some(existing_staff)) = self.repo.find_by_id(id).await {
                    if existing_staff.email != *email {
                        if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                            return Err(format!("Staff email already exists: {}", email));
                        }
                    }
                }
            }
        }

        // Perform bulk update
        let updated_staff = self
            .repo
            .update_many_school_staff(updates)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_staff)
    }

    /// Bulk update active status for multiple school staff members
    pub async fn bulk_update_active_status(
        &self,
        request: &BulkUpdateActiveStatusRequest,
    ) -> Result<Vec<SchoolStaff>, String> {
        let updated_staff = self
            .repo
            .bulk_update_active_status(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_staff)
    }

    /// Bulk add tags to multiple school staff members
    pub async fn bulk_add_tags(
        &self,
        request: &BulkTagsRequest,
    ) -> Result<Vec<SchoolStaff>, String> {
        let updated_staff = self
            .repo
            .bulk_add_tags(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_staff)
    }

    /// Bulk remove tags from multiple school staff members
    pub async fn bulk_remove_tags(
        &self,
        request: &BulkTagsRequest,
    ) -> Result<Vec<SchoolStaff>, String> {
        let updated_staff = self
            .repo
            .bulk_remove_tags(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_staff)
    }

    /// Delete multiple school staff members
    pub async fn delete_many_school_staff(&self, request: &BulkIdsRequest) -> Result<u64, String> {
        self.repo
            .delete_many_school_staff(request)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // 🔔 EVENT BROADCASTING METHODS
    // ------------------------------------------------------------------

    /// Broadcast school staff update event
    async fn broadcast_school_staff_update(state: &web::Data<AppState>, staff_id: &ObjectId) {
        let state_clone = state.clone();
        let staff_id_clone = *staff_id;

        actix_rt::spawn(async move {
            // Fetch the updated school staff with relations for broadcasting
            let repo = SchoolStaffRepo::new(&state_clone.db.main_db());
            if let Ok(Some(updated_staff)) = repo
                .find_by_id_with_relations(&IdType::from_object_id(staff_id_clone))
                .await
            {
                EventService::broadcast_updated(
                    &state_clone,
                    "school_staff",
                    &staff_id_clone.to_hex(),
                    &updated_staff,
                )
                .await;
            }
        });
    }

    /// Broadcast school staff deletion event
    async fn broadcast_school_staff_deletion(
        state: &web::Data<AppState>,
        staff_id: &ObjectId,
        staff: &SchoolStaff,
    ) {
        let state_clone = state.clone();
        let staff_id_clone = *staff_id;
        let staff_clone = staff.clone();

        actix_rt::spawn(async move {
            EventService::broadcast_deleted(
                &state_clone,
                "school_staff",
                &staff_id_clone.to_hex(),
                &staff_clone,
            )
            .await;
        });
    }

    // ------------------------------------------------------------------
    // 🔧 UTILITY METHODS
    // ------------------------------------------------------------------

    /// Prepare school staff members for bulk creation
    pub fn prepare_school_staff_for_bulk_creation(
        &self,
        staff_members: Vec<SchoolStaff>,
        school_id: Option<ObjectId>,
        creator_id: Option<ObjectId>,
    ) -> Result<Vec<SchoolStaff>, String> {
        let prepared_staff: Vec<SchoolStaff> = staff_members
            .into_iter()
            .map(|mut staff| {
                if let Some(sid) = school_id {
                    staff.school_id = Some(sid);
                }
                if let Some(cid) = creator_id {
                    staff.creator_id = Some(cid);
                }
                staff
            })
            .collect();

        Ok(prepared_staff)
    }

    /// Get director for a school
    pub async fn get_school_director(
        &self,
        school_id: &IdType,
    ) -> Result<Option<SchoolStaff>, String> {
        let directors = self
            .repo
            .find_by_school_and_type(school_id, SchoolStaffType::Director)
            .await
            .map_err(|e| e.message)?;

        // Return the first director found (assuming one director per school)
        Ok(directors.into_iter().next())
    }

    /// Get head of studies for a school
    pub async fn get_head_of_studies(
        &self,
        school_id: &IdType,
    ) -> Result<Option<SchoolStaff>, String> {
        let heads = self
            .repo
            .find_by_school_and_type(school_id, SchoolStaffType::HeadOfStudies)
            .await
            .map_err(|e| e.message)?;

        // Return the first head of studies found
        Ok(heads.into_iter().next())
    }

    /// Check if a user is staff member of a specific school
    pub async fn is_user_school_staff(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<bool, String> {
        let staff = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message)?;

        if let Some(staff_member) = staff {
            // Compare school IDs
            if let Some(staff_school_id) = staff_member.school_id {
                let target_school_id = parse_object_id(school_id)?;
                return Ok(staff_school_id == target_school_id);
            }
        }

        Ok(false)
    }

    /// Check if a user has a specific staff type in a school
    pub async fn has_staff_type(
        &self,
        user_id: &IdType,
        school_id: &IdType,
        staff_type: SchoolStaffType,
    ) -> Result<bool, String> {
        let staff = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message)?;

        if let Some(staff_member) = staff {
            // Check school ID and staff type
            if let Some(staff_school_id) = staff_member.school_id {
                let target_school_id = parse_object_id(school_id)?;

                if staff_school_id == target_school_id && staff_member.r#type == staff_type {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
