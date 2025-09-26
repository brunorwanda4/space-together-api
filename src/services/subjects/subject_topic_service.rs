use crate::{
    domain::subjects::subject_topic::{SubjectTopic, UpdateSubjectTopic},
    models::id_model::IdType,
    repositories::subjects::subject_topic_repo::SubjectTopicRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SubjectTopicService<'a> {
    repo: &'a SubjectTopicRepo,
}

impl<'a> SubjectTopicService<'a> {
    pub fn new(repo: &'a SubjectTopicRepo) -> Self {
        Self { repo }
    }

    /// Get all topics
    pub async fn get_all_topics(&self) -> Result<Vec<SubjectTopic>, String> {
        self.repo.get_all_topics().await.map_err(|e| e.message)
    }

    /// Create a new topic
    pub async fn create_topic(&self, mut new_topic: SubjectTopic) -> Result<SubjectTopic, String> {
        // ✅ Ensure Mongo generates ID
        new_topic.id = Some(ObjectId::new());

        // Set timestamps
        let now = Some(Utc::now());
        new_topic.created_at = now;
        new_topic.updated_at = now;

        // Save topic in database
        let inserted_topic = self
            .repo
            .insert_topic(&new_topic)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted_topic)
    }

    /// Get topic by ID
    pub async fn get_topic_by_id(&self, id: &IdType) -> Result<SubjectTopic, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Topic not found".to_string())
    }

    /// Update a topic by id
    pub async fn update_topic(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectTopic,
    ) -> Result<SubjectTopic, String> {
        let mut topic_to_update = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Topic not found".to_string())?;

        // ✅ Only overwrite if provided
        if let Some(ref title) = updated_data.title {
            topic_to_update.title = title.to_string();
        }
        if let Some(ref description) = updated_data.description {
            topic_to_update.description = Some(description.to_string());
        }
        if let Some(order) = updated_data.order {
            topic_to_update.order = order;
        }
        if let Some(lo_id) = updated_data.learning_outcome_id.clone() {
            topic_to_update.learning_outcome_id = Some(lo_id);
        }
        if let Some(parent_id) = updated_data.parent_topic_id.clone() {
            topic_to_update.parent_topic_id = Some(parent_id);
        }

        topic_to_update.updated_at = Some(Utc::now());

        let updated_topic = self
            .repo
            .update_topic(id, &updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_topic)
    }

    /// Delete a topic by id
    pub async fn delete_topic(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_topic(id).await.map_err(|e| e.message)
    }
}
