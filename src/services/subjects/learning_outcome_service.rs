use crate::{
    domain::subjects::learning_outcome::{
        LearningOutcome, LearningOutcomeWithOthers, UpdateLearningOutcome,
    },
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

    pub async fn get_outcome_with_topics_by_id(
        &self,
        id: &IdType,
    ) -> Result<LearningOutcomeWithOthers, String> {
        self.repo
            .find_by_id_with_topics(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning outcome not found".to_string())
    }

    /// ✅ Get all learning outcomes for a subject with topics + materials
    pub async fn get_outcomes_with_topics_by_subject(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<LearningOutcomeWithOthers>, String> {
        self.repo
            .find_by_subject_with_topics(subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get by subject id
    pub async fn get_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<LearningOutcome>, String> {
        self.repo
            .find_by_subject_id(subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Create a new learning outcome
    pub async fn create_outcome(
        &self,
        mut new_outcome: LearningOutcome,
    ) -> Result<LearningOutcome, String> {
        // 🚫 Prevent duplicate titles within same subject
        if let Ok(Some(_)) = self.repo.find_by_title(&new_outcome.title).await {
            return Err("Learning outcome title already exists".to_string());
        }

        // 🚫 Prevent invalid order
        if new_outcome.order <= 0 {
            return Err("Order must be greater than 0".to_string());
        }

        // 🚫 Prevent duplicate order for the same subject
        if let Some(subject_id) = &new_outcome.subject_id {
            if let Ok(Some(_)) = self.repo.find_by_order(subject_id, new_outcome.order).await {
                return Err(format!(
                    "Learning outcome order {} already exists for this subject",
                    new_outcome.order
                ));
            }
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
        // Fetch existing outcome
        let outcome = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning outcome not found".to_string())?;

        // 🚫 Prevent duplicate titles
        if let Some(ref new_title) = updated_data.title {
            if outcome.title != *new_title {
                if let Ok(Some(_)) = self.repo.find_by_title(new_title).await {
                    return Err("Learning outcome title already exists".to_string());
                }
            }
        }

        // 🚫 Prevent invalid order
        if let Some(new_order) = updated_data.order {
            if new_order <= 0 {
                return Err("Order must be greater than 0".to_string());
            }

            if let Some(subject_id) = outcome.subject_id {
                if let Ok(Some(existing)) = self.repo.find_by_order(&subject_id, new_order).await {
                    // Make sure it’s not the same record
                    if existing.id != outcome.id {
                        return Err(format!(
                            "Learning outcome order {} already exists for this subject",
                            new_order
                        ));
                    }
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
