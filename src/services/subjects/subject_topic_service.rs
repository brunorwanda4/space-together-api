use chrono::Utc;
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::subjects::subject_topic::{SubjectTopic, UpdateSubjectTopic},
    models::id_model::IdType,
    repositories::subjects::learning_outcome_repo::LearningOutcomeRepo,
    repositories::subjects::subject_topic_repo::SubjectTopicRepo,
    services::event_service::EventService,
};
use actix_web::web;

pub struct SubjectTopicService<'a> {
    repo: &'a SubjectTopicRepo,
}

impl<'a> SubjectTopicService<'a> {
    pub fn new(repo: &'a SubjectTopicRepo) -> Self {
        Self { repo }
    }

    // ------------------------------------------------------------------
    // ✅ CRUD OPERATIONS
    // ------------------------------------------------------------------

    pub async fn get_all_topics(&self) -> Result<Vec<SubjectTopic>, String> {
        self.repo.get_all_topics().await.map_err(|e| e.message)
    }

    pub async fn get_topic_by_id(&self, id: &IdType) -> Result<SubjectTopic, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Topic not found".to_string())
    }

    pub async fn get_topics_by_learning_outcome_and_parent(
        &self,
        learning_outcome_id: &IdType,
        parent_topic_id: Option<&IdType>,
    ) -> Result<Vec<SubjectTopic>, String> {
        self.repo
            .get_topics_by_learning_outcome_and_parent(learning_outcome_id, parent_topic_id)
            .await
            .map_err(|e| e.message)
    }

    pub async fn delete_topic(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_topic(id).await.map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ CREATE
    // ------------------------------------------------------------------

    pub async fn create_topic(&self, mut new_topic: SubjectTopic) -> Result<SubjectTopic, String> {
        // ✅ First, check duplicates manually before insert
        if let Some(lo_id) = &new_topic.learning_outcome_id {
            // Check if same order exists in this learning outcome
            if self
                .repo
                .exists_order_in_learning_outcome(&IdType::ObjectId(*lo_id), new_topic.order)
                .await
                .map_err(|e| e.message.clone())?
            {
                return Err(format!(
                    "A topic with order {} already exists for this learning outcome.",
                    new_topic.order
                ));
            }
        }

        if let Some(parent_id) = &new_topic.parent_topic_id {
            // Check if same order exists under same parent
            if self
                .repo
                .exists_order_in_parent_topic(&IdType::ObjectId(*parent_id), new_topic.order)
                .await
                .map_err(|e| e.message.clone())?
            {
                return Err(format!(
                    "A topic with order {} already exists under this parent topic.",
                    new_topic.order
                ));
            }
        }

        // ✅ If no duplicates found, continue insert
        new_topic.id = Some(ObjectId::new());
        let now = Some(Utc::now());
        new_topic.created_at = now;
        new_topic.updated_at = now;

        self.repo
            .insert_topic(&new_topic)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ CREATE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn create_topic_with_events(
        &self,
        new_topic: SubjectTopic,
        state: &web::Data<AppState>,
    ) -> Result<SubjectTopic, String> {
        let topic = self.create_topic(new_topic).await?;

        // 🔔 Broadcast learning outcome update if this topic belongs to one
        if let Some(learning_outcome_id) = &topic.learning_outcome_id {
            Self::broadcast_learning_outcome_update(state, learning_outcome_id).await;
        }

        Ok(topic)
    }

    // ------------------------------------------------------------------
    // ✅ UPDATE
    // ------------------------------------------------------------------

    pub async fn update_topic(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectTopic,
    ) -> Result<SubjectTopic, String> {
        let mut existing = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Topic not found".to_string())?;

        let final_order = updated_data.order.unwrap_or(existing.order);
        let final_lo_id = updated_data
            .learning_outcome_id
            .or(existing.learning_outcome_id);
        let final_parent_id = updated_data.parent_topic_id.or(existing.parent_topic_id);

        if let Some(lo_id) = &final_lo_id {
            self.ensure_unique_order_in_learning_outcome(lo_id, final_order, Some(id))
                .await?;
        }
        if let Some(parent_id) = &final_parent_id {
            self.ensure_unique_order_in_parent_topic(parent_id, final_order, Some(id))
                .await?;
        }

        // Use ref bindings to avoid moving out of updated_data
        if let Some(ref title) = updated_data.title {
            existing.title = title.clone();
        }
        if let Some(ref desc) = updated_data.description {
            existing.description = Some(desc.clone());
        }
        if let Some(order) = updated_data.order {
            existing.order = order;
        }
        if let Some(lo_id) = updated_data.learning_outcome_id {
            existing.learning_outcome_id = Some(lo_id);
        }
        if let Some(parent_id) = updated_data.parent_topic_id {
            existing.parent_topic_id = Some(parent_id);
        }

        existing.updated_at = Some(Utc::now());

        self.repo
            .update_topic(id, &updated_data)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ UPDATE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn update_topic_with_events(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectTopic,
        state: &web::Data<AppState>,
    ) -> Result<SubjectTopic, String> {
        let old_topic = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or("Topic not found")?;
        let updated_topic = self.update_topic(id, updated_data).await?;

        // 🔔 Get all affected learning outcome IDs
        let affected_lo_ids = self
            .get_affected_learning_outcome_ids(&old_topic, &updated_topic)
            .await?;

        // Broadcast updates for all affected learning outcomes
        for lo_id in affected_lo_ids {
            Self::broadcast_learning_outcome_update(state, &lo_id).await;
        }

        Ok(updated_topic)
    }

    // ------------------------------------------------------------------
    // ✅ DELETE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn delete_topic_with_events(
        &self,
        id: &IdType,
        state: &web::Data<AppState>,
    ) -> Result<(), String> {
        let topic = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or("Topic not found")?;

        // Store learning outcome ID before deletion
        let learning_outcome_id = topic.learning_outcome_id;

        self.delete_topic(id).await?;

        // 🔔 Broadcast learning outcome update if this topic belonged to one
        if let Some(lo_id) = learning_outcome_id {
            Self::broadcast_learning_outcome_update(state, &lo_id).await;
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // 🔧 HELPERS
    // ------------------------------------------------------------------

    async fn ensure_unique_order_in_learning_outcome(
        &self,
        lo_id: &ObjectId,
        order: f32,
        exclude_id: Option<&IdType>,
    ) -> Result<(), String> {
        if !self
            .repo
            .exists_order_in_learning_outcome(&IdType::ObjectId(*lo_id), order)
            .await
            .map_err(|e| e.message.clone())?
        {
            return Ok(());
        }

        let topics = self
            .repo
            .get_topics_by_learning_outcome_and_parent(&IdType::ObjectId(*lo_id), None)
            .await
            .map_err(|e| e.message.clone())?;

        if let Some(dup) = self.find_duplicate_topic(&topics, order, exclude_id)? {
            Err(format!(
                "Order {} already exists for this learning outcome (topic: '{}')",
                order, dup.title
            ))
        } else {
            Ok(())
        }
    }

    async fn ensure_unique_order_in_parent_topic(
        &self,
        parent_id: &ObjectId,
        order: f32,
        exclude_id: Option<&IdType>,
    ) -> Result<(), String> {
        if !self
            .repo
            .exists_order_in_parent_topic(&IdType::ObjectId(*parent_id), order)
            .await
            .map_err(|e| e.message.clone())?
        {
            return Ok(());
        }

        let topics = self
            .repo
            .get_topics_by_learning_outcome_and_parent(
                &IdType::from_object_id(*parent_id),
                Some(&IdType::from_object_id(*parent_id)),
            )
            .await
            .map_err(|e| e.message.clone())?;

        if let Some(dup) = self.find_duplicate_topic(&topics, order, exclude_id)? {
            Err(format!(
                "Order {} already exists for this learning outcome (topic: '{}')",
                order, dup.title
            ))
        } else {
            Ok(())
        }
    }

    fn find_duplicate_topic<'b>(
        &self,
        topics: &'b [SubjectTopic],
        order: f32,
        exclude_id: Option<&IdType>,
    ) -> Result<Option<&'b SubjectTopic>, String> {
        let exclude_oid = exclude_id
            .map(|id| id.to_object_id().map_err(|e| e.message.clone()))
            .transpose()?;

        Ok(topics.iter().find(|t| {
            (t.order - order).abs() < f32::EPSILON
                && exclude_oid.as_ref().map_or(true, |oid| t.id != Some(*oid))
        }))
    }

    async fn get_affected_learning_outcome_ids(
        &self,
        old_topic: &SubjectTopic,
        new_topic: &SubjectTopic,
    ) -> Result<Vec<ObjectId>, String> {
        let mut affected_ids = Vec::new();

        // Add old learning outcome ID if it changed
        if let Some(old_lo_id) = old_topic.learning_outcome_id {
            affected_ids.push(old_lo_id);
        }

        // Add new learning outcome ID if it's different
        if let Some(new_lo_id) = new_topic.learning_outcome_id {
            if Some(new_lo_id) != old_topic.learning_outcome_id {
                affected_ids.push(new_lo_id);
            }
        }

        // Remove duplicates
        affected_ids.sort();
        affected_ids.dedup();

        Ok(affected_ids)
    }

    async fn broadcast_learning_outcome_update(
        state: &web::Data<AppState>,
        learning_outcome_id: &ObjectId,
    ) {
        let state_clone = state.clone();
        let lo_id_clone = *learning_outcome_id;

        actix_rt::spawn(async move {
            // Fetch the updated learning outcome with all its topics and materials
            let repo = LearningOutcomeRepo::new(&state_clone.db);
            if let Ok(Some(updated_lo)) = repo
                .find_by_id_with_topics(&IdType::from_object_id(lo_id_clone))
                .await
            {
                EventService::broadcast_updated(
                    &state_clone,
                    "learning_outcome",
                    &lo_id_clone.to_hex(),
                    &updated_lo,
                )
                .await;
            }
        });
    }
}
