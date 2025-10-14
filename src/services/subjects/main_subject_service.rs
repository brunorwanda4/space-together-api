use crate::{
    domain::subjects::main_subject::{MainSubject, MainSubjectWithOthers, UpdateMainSubject},
    models::id_model::IdType,
    repositories::subjects::main_subject_repo::MainSubjectRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct MainSubjectService<'a> {
    repo: &'a MainSubjectRepo,
}

impl<'a> MainSubjectService<'a> {
    pub fn new(repo: &'a MainSubjectRepo) -> Self {
        Self { repo }
    }
    // get by main class id
    pub async fn get_subjects_by_main_class_id(
        &self,
        id: &IdType,
    ) -> Result<Vec<MainSubject>, String> {
        self.repo
            .find_by_main_class_id(id)
            .await
            .map_err(|e| e.message.clone())
    }
    // get main subject and other with main subject id
    pub async fn get_subjects_with_others_by_main_subject_id(
        &self,
        id: &IdType,
    ) -> Result<Option<MainSubjectWithOthers>, String> {
        self.repo
            .find_by_id_with_others(id)
            .await
            .map_err(|e| e.message.clone())
    }

    /// Get all subjects
    pub async fn get_all_subjects(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        is_active: Option<bool>,
    ) -> Result<Vec<MainSubject>, String> {
        self.repo
            .get_all_subjects(filter, limit, skip, is_active)
            .await
            .map_err(|e| e.message)
    }

    pub async fn get_all_subjects_with_others(&self) -> Result<Vec<MainSubjectWithOthers>, String> {
        self.repo
            .get_all_subjects_with_others()
            .await
            .map_err(|e| e.message)
    }

    /// Create a new subject
    pub async fn create_subject(
        &self,
        mut new_subject: MainSubject,
    ) -> Result<MainSubject, String> {
        // ✅ Check if subject code already exists
        if let Ok(Some(_)) = self.repo.find_by_code(&new_subject.code).await {
            return Err("Subject code already exists".to_string());
        }

        let now = Some(Utc::now());
        new_subject.created_at = now;
        new_subject.updated_at = now;

        // Ensure Mongo generates id
        new_subject.id = Some(ObjectId::new());

        let inserted_subject = self
            .repo
            .insert_subject(&new_subject)
            .await
            .map_err(|e| e.message)?;
        Ok(inserted_subject)
    }

    /// Get subject by ID
    pub async fn get_subject_by_id(&self, id: &IdType) -> Result<MainSubject, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Main subject not found".to_string())
    }

    /// Get subject by code
    pub async fn get_subject_by_code(&self, code: &str) -> Result<MainSubject, String> {
        self.repo
            .find_by_code(code)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Main subject not found".to_string())
    }

    /// Update a subject by id
    pub async fn update_subject(
        &self,
        id: &IdType,
        updated_data: UpdateMainSubject,
    ) -> Result<MainSubject, String> {
        // Fetch subject
        let subject = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Main subject not found".to_string())?;

        // Prevent duplicate codes if code is being updated
        if let Some(ref new_code) = updated_data.code {
            if subject.code != *new_code {
                if let Ok(Some(_)) = self.repo.find_by_code(new_code).await {
                    return Err("Subject code already exists".to_string());
                }
            }
        }

        let updated_subject = self
            .repo
            .update_subject(id, updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_subject)
    }

    /// Delete a subject by id
    pub async fn delete_subject(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_subject(id).await.map_err(|e| e.message)
    }
}
