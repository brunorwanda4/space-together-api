use crate::{
    domain::school::{School, SchoolStats, UpdateSchool},
    models::id_model::IdType,
    repositories::school_repo::SchoolRepo,
    services::cloudinary_service::CloudinaryService,
    utils::{
        code::generate_code,
        names::{is_valid_name, is_valid_username},
        school_utils::{sanitize_school, sanitize_schools},
    },
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SchoolService<'a> {
    repo: &'a SchoolRepo,
}

impl<'a> SchoolService<'a> {
    pub fn new(repo: &'a SchoolRepo) -> Self {
        Self { repo }
    }

    /// Get all schools
    pub async fn get_all_schools(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<School>, String> {
        let schools = self
            .repo
            .get_all_schools(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_schools(schools))
    }

    /// Get school statistics
    pub async fn get_school_stats(&self) -> Result<SchoolStats, String> {
        self.repo.get_school_stats().await.map_err(|e| e.message)
    }

    /// Create a new school
    pub async fn create_school(&self, mut new_school: School) -> Result<School, String> {
        is_valid_username(&new_school.username)?;
        is_valid_name(&new_school.name)?;

        // Check if code already exists
        if let Some(school_code) = new_school.code {
            if let Ok(Some(_)) = self.repo.find_by_code(&school_code).await {
                return Err("School code already exists".to_string());
            }
        }

        // Check if username already exists
        if let Ok(Some(_)) = self.repo.find_by_username(&new_school.username).await {
            return Err("School username already exists".to_string());
        }

        // Handle logo upload
        if let Some(logo_file) = new_school.logo.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&logo_file).await?;
            new_school.logo_id = Some(cloud_res.public_id);
            new_school.logo = Some(cloud_res.secure_url);
        }

        new_school.code = Some(generate_code());
        // Set timestamps
        let now = Some(Utc::now());
        new_school.created_at = now;
        new_school.updated_at = now;

        // Set default values for optional fields
        if new_school.is_active.is_none() {
            new_school.is_active = Some(false);
        }

        let school_id = ObjectId::new();
        new_school.id = Some(school_id);

        // let school_db = self.repo.

        // Save school in database
        let inserted_school = self
            .repo
            .insert_school(&new_school)
            .await
            .map_err(|e| e.message)?;

        Ok(sanitize_school(inserted_school))
    }

    /// Get school by ID
    pub async fn get_school_by_id(&self, id: &IdType) -> Result<School, String> {
        let school = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School not found".to_string())?;

        Ok(sanitize_school(school))
    }

    /// Get school by username
    pub async fn get_school_by_username(&self, username: &str) -> Result<School, String> {
        let school = self
            .repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School not found".to_string())?;

        Ok(sanitize_school(school))
    }

    /// Get school by code
    pub async fn get_school_by_code(&self, code: &str) -> Result<School, String> {
        let school = self
            .repo
            .find_by_code(code)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School not found".to_string())?;

        Ok(sanitize_school(school))
    }

    /// Alternative approach if you want to update the complete school like user service
    pub async fn update_school(
        &self,
        id: &IdType,
        updated_data: UpdateSchool,
    ) -> Result<School, String> {
        // Validate username if provided
        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }
        if let Some(ref name) = updated_data.name {
            is_valid_name(name)?;
        }

        let existing_school = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School not found".to_string())?;

        // Check username uniqueness if provided and changed
        if let Some(ref username) = updated_data.username {
            if existing_school.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("School username already exists".to_string());
                }
            }
        }

        // Check code uniqueness if provided and changed
        if let Some(ref code) = updated_data.code {
            if existing_school.code.as_ref() != Some(code) {
                if let Ok(Some(_)) = self.repo.find_by_code(code).await {
                    return Err("School code already exists".to_string());
                }
            }
        }

        // Create a complete School object by merging existing data with updates
        let mut merged_school = existing_school.clone();

        // Handle logo update if provided
        if let Some(ref new_logo) = updated_data.logo {
            if let Some(old_logo_id) = merged_school.logo_id.clone() {
                CloudinaryService::delete_from_cloudinary(&old_logo_id)
                    .await
                    .ok();
            }
            let cloud_res = CloudinaryService::upload_to_cloudinary(new_logo).await?;
            merged_school.logo_id = Some(cloud_res.public_id);
            merged_school.logo = Some(cloud_res.secure_url);
        }

        // Update only the fields that are provided in updated_data
        if let Some(username) = updated_data.username {
            merged_school.username = username;
        }

        if let Some(database_name) = updated_data.database_name {
            merged_school.database_name = Some(database_name);
        }

        if let Some(name) = updated_data.name {
            merged_school.name = name;
        }
        if let Some(code) = updated_data.code {
            merged_school.code = Some(code);
        }
        if let Some(description) = updated_data.description {
            merged_school.description = Some(description);
        }
        if let Some(school_type) = updated_data.school_type {
            merged_school.school_type = Some(school_type);
        }
        if let Some(curriculum) = updated_data.curriculum {
            merged_school.curriculum = Some(curriculum);
        }
        if let Some(education_level) = updated_data.education_level {
            merged_school.education_level = Some(education_level);
        }
        if let Some(accreditation_number) = updated_data.accreditation_number {
            merged_school.accreditation_number = Some(accreditation_number);
        }
        if let Some(affiliation) = updated_data.affiliation {
            merged_school.affiliation = Some(affiliation);
        }
        if let Some(school_members) = updated_data.school_members {
            merged_school.school_members = Some(school_members);
        }
        if let Some(address) = updated_data.address {
            merged_school.address = Some(address);
        }
        if let Some(contact) = updated_data.contact {
            merged_school.contact = Some(contact);
        }
        if let Some(website) = updated_data.website {
            merged_school.website = Some(website);
        }
        if let Some(social_media) = updated_data.social_media {
            merged_school.social_media = Some(social_media);
        }
        if let Some(student_capacity) = updated_data.student_capacity {
            merged_school.student_capacity = Some(student_capacity);
        }
        if let Some(uniform_required) = updated_data.uniform_required {
            merged_school.uniform_required = Some(uniform_required);
        }
        if let Some(attendance_system) = updated_data.attendance_system {
            merged_school.attendance_system = Some(attendance_system);
        }
        if let Some(scholarship_available) = updated_data.scholarship_available {
            merged_school.scholarship_available = Some(scholarship_available);
        }
        if let Some(classrooms) = updated_data.classrooms {
            merged_school.classrooms = Some(classrooms);
        }
        if let Some(library) = updated_data.library {
            merged_school.library = Some(library);
        }
        if let Some(labs) = updated_data.labs {
            merged_school.labs = Some(labs);
        }
        if let Some(sports_extracurricular) = updated_data.sports_extracurricular {
            merged_school.sports_extracurricular = Some(sports_extracurricular);
        }
        if let Some(online_classes) = updated_data.online_classes {
            merged_school.online_classes = Some(online_classes);
        }
        if let Some(is_active) = updated_data.is_active {
            merged_school.is_active = Some(is_active);
        }

        merged_school.updated_at = Some(Utc::now());

        let updated_school = self
            .repo
            .update_school(id, &merged_school)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_school(updated_school))
    }

    /// Update a school by id (partial update using repository method)
    pub async fn update_school_partial(
        &self,
        id: &IdType,
        mut updated_data: UpdateSchool,
    ) -> Result<School, String> {
        // Validate username if provided
        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }
        if let Some(ref name) = updated_data.name {
            is_valid_name(name)?;
        }

        let existing_school = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "School not found".to_string())?;

        // Check username uniqueness if provided and changed
        if let Some(ref username) = updated_data.username {
            if existing_school.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("School username already exists".to_string());
                }
            }
        }

        // Check code uniqueness if provided and changed
        if let Some(ref code) = updated_data.code {
            if existing_school.code.as_ref() != Some(code) {
                if let Ok(Some(_)) = self.repo.find_by_code(code).await {
                    return Err("School code already exists".to_string());
                }
            }
        }

        // Handle logo update if provided
        if let Some(ref new_logo) = updated_data.logo {
            if let Some(old_logo_id) = existing_school.logo_id.clone() {
                CloudinaryService::delete_from_cloudinary(&old_logo_id)
                    .await
                    .ok();
            }
            let cloud_res = CloudinaryService::upload_to_cloudinary(new_logo).await?;
            updated_data.logo_id = Some(cloud_res.public_id);
            updated_data.logo = Some(cloud_res.secure_url);
        }

        updated_data.updated_at = Some(Utc::now());

        let updated_school = self
            .repo
            .update_school_partial(id, updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(sanitize_school(updated_school))
    }

    /// Delete a school by id
    pub async fn delete_school(&self, id: &IdType) -> Result<(), String> {
        let school = self.repo.find_by_id(id).await.map_err(|e| e.message)?;

        if let Some(delete_school) = school {
            // Delete logo from cloudinary if exists
            if let Some(logo_public_id) = delete_school.logo_id {
                CloudinaryService::delete_from_cloudinary(&logo_public_id)
                    .await
                    .ok(); // ignore delete errors
            }
        }

        self.repo.delete_school(id).await.map_err(|e| e.message)
    }
}
