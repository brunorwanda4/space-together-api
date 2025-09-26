use crate::{
    domain::subjects::learning_outcome::{LearningOutcome, UpdateLearningOutcome},
    models::id_model::IdType,
    repositories::subjects::learning_outcome_repo::LearningOutcomeRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct LearningOutcomeService<'a> {
    repo: &'a LearningOutcomeRepo,
}

impl<'a> LearningOutcomeService<'a> {
    pub fn new(repo: &'a LearningOutcomeRepo) -> Self {
        Self { repo }
    }

    /// Get all learning outcomes
    pub async fn get_all_outcomes(&self) -> Result<Vec<LearningOutcome>, String> {
        self.repo.get_all_outcomes().await.map_err(|e| e.message)
    }

    /// Create a new learning outcome
    pub async fn create_outcome(
        &self,
        mut new_outcome: LearningOutcome,
    ) -> Result<LearningOutcome, String> {
        // ✅ Check if outcome title already exists for the same subject
        if let Ok(Some(_)) = self.repo.find_by_title(&new_outcome.title).await {
            return Err("Learning outcome title already exists".to_string());
        }

        let now = Some(Utc::now());
        new_outcome.created_at = now;
        new_outcome.updated_at = now;

        // Ensure Mongo generates id
        new_outcome.id = Some(ObjectId::new());

        let inserted_outcome = self
            .repo
            .insert_outcome(&new_outcome)
            .await
            .map_err(|e| e.message)?;
        Ok(inserted_outcome)
    }

    /// Get learning outcome by ID
    pub async fn get_outcome_by_id(&self, id: &IdType) -> Result<LearningOutcome, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning outcome not found".to_string())
    }

    /// Get learning outcome by title
    pub async fn get_outcome_by_title(&self, title: &str) -> Result<LearningOutcome, String> {
        self.repo
            .find_by_title(title)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning outcome not found".to_string())
    }

    /// Update a learning outcome by id
    pub async fn update_outcome(
        &self,
        id: &IdType,
        updated_data: UpdateLearningOutcome,
    ) -> Result<LearningOutcome, String> {
        // Fetch learning outcome
        let outcome = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning outcome not found".to_string())?;

        // Prevent duplicate titles if title is being updated
        if let Some(ref new_title) = updated_data.title {
            if outcome.title != *new_title {
                if let Ok(Some(_)) = self.repo.find_by_title(new_title).await {
                    return Err("Learning outcome title already exists".to_string());
                }
            }
        }

        let updated_outcome = self
            .repo
            .update_outcome(id, updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_outcome)
    }

    /// Delete a learning outcome by id
    pub async fn delete_outcome(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_outcome(id).await.map_err(|e| e.message)
    }
}
