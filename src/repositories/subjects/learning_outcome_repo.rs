use crate::domain::subjects::learning_outcome::{
    LearningOutcome, LearningOutcomeWithOthers, UpdateLearningOutcome,
};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::helpers::subjects_helpers::subject_topic_helper::sort_learning_outcome_topics_typed;
use crate::models::id_model::IdType;
use crate::pipeline::learning_outcome_pipeline::learning_outcome_with_topics_pipeline;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{self, doc, oid::ObjectId, Document};
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};

pub struct LearningOutcomeRepo {
    pub collection: Collection<LearningOutcome>,
}

impl LearningOutcomeRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<LearningOutcome>("learning_outcomes"),
        }
    }

    pub async fn get_all_outcomes(&self) -> Result<Vec<LearningOutcome>, AppError> {
        let pipeline = vec![doc! { "$sort": { "order": 1 } }];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch learning outcomes: {}", e),
            })?;

        let mut outcomes = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning outcomes: {}", e),
        })? {
            let outcome: LearningOutcome = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize learning outcome: {}", e),
            })?;
            outcomes.push(outcome);
        }

        Ok(outcomes)
    }

    /// ✅ Get one learning outcome with all topics + topic materials
    pub async fn find_by_id_with_topics(
        &self,
        id: &IdType,
    ) -> Result<Option<LearningOutcomeWithOthers>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Invalid ObjectId: {}", e),
        })?;

        // Run aggregation using the fixed helper
        let mut outcome: Option<LearningOutcomeWithOthers> = aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            learning_outcome_with_topics_pipeline(doc! { "_id": obj_id }),
        )
        .await?;

        // Sort topics in-place
        if let Some(ref mut outcome_value) = outcome {
            if let Some(ref mut topics) = outcome_value.topics {
                sort_learning_outcome_topics_typed(topics);
            }
        }

        Ok(outcome)
    }

    /// ✅ Get all learning outcomes for a subject with topics and materials
    pub async fn find_by_subject_with_topics(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<LearningOutcomeWithOthers>, AppError> {
        let obj_id = ObjectId::parse_str(subject_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid ObjectId: {}", e),
        })?;

        // Run aggregation using fixed helper
        let mut outcomes: Vec<LearningOutcomeWithOthers> = aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            learning_outcome_with_topics_pipeline(doc! { "subject_id": obj_id }),
        )
        .await?;

        // Sort topics in-place for each outcome
        for outcome in &mut outcomes {
            if let Some(ref mut topics) = outcome.topics {
                sort_learning_outcome_topics_typed(topics);
            }
        }

        Ok(outcomes)
    }

    /// Find by id
    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<LearningOutcome>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find learning outcome by id: {}", e),
            })
    }

    /// Find by title
    pub async fn find_by_title(&self, title: &str) -> Result<Option<LearningOutcome>, AppError> {
        self.collection
            .find_one(doc! { "title": title })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find learning outcome by title: {}", e),
            })
    }

    /// Find all learning outcomes by subject_id (sorted by order ascending)
    pub async fn find_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<LearningOutcome>, AppError> {
        let obj_id = ObjectId::parse_str(subject_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;
        let mut cursor = self
            .collection
            .find(doc! { "subject_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch learning outcomes by subject_id: {}", e),
            })?;

        let mut outcomes = Vec::new();
        while let Some(outcome) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning outcomes by subject_id: {}", e),
        })? {
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    /// Find by subject_id and order
    pub async fn find_by_order(
        &self,
        subject_id: &ObjectId,
        order: i32,
    ) -> Result<Option<LearningOutcome>, AppError> {
        self.collection
            .find_one(doc! { "subject_id": subject_id, "order": order })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find learning outcome by order: {}", e),
            })
    }

    /// Insert new learning outcome
    pub async fn insert_outcome(
        &self,
        outcome: &LearningOutcome,
    ) -> Result<LearningOutcome, AppError> {
        // Create unique index on subject_id + order
        let order_index = IndexModel::builder()
            .keys(doc! { "subject_id": 1, "order": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(order_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create unique order index: {}", e),
            })?;

        let mut outcome_to_insert = outcome.clone();
        outcome_to_insert.id = None;
        outcome_to_insert.created_at = Some(Utc::now());
        outcome_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&outcome_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert learning outcome: {}", e),
            })?;

        let inserted_id: ObjectId = res.inserted_id.as_object_id().ok_or(AppError {
            message: "Failed to get inserted learning outcome id".to_string(),
        })?;

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Learning outcome not found after insert".to_string(),
            })
    }

    /// Update learning outcome
    pub async fn update_outcome(
        &self,
        id: &IdType,
        update: UpdateLearningOutcome,
    ) -> Result<LearningOutcome, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(&update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        // Remove `_id` and null values
        update_doc = update_doc
            .into_iter()
            .filter(|(k, v)| k != "_id" && !matches!(v, bson::Bson::Null))
            .collect();

        // Always refresh updated_at
        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        self.collection
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update learning outcome: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Learning outcome not found after update".to_string(),
        })
    }

    /// Delete learning outcome
    pub async fn delete_outcome(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete learning outcome: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No learning outcome deleted; outcome may not exist".to_string(),
            });
        }

        Ok(())
    }
}
