use crate::domain::subjects::learning_outcome::{LearningOutcome, UpdateLearningOutcome};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct LearningOutcomeRepo {
    pub collection: Collection<LearningOutcome>,
}

impl LearningOutcomeRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<LearningOutcome>("learning_outcomes"),
        }
    }

    /// Find by id
    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<LearningOutcome>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find learning outcome by id: {}", e),
            })
    }

    /// Find by title
    pub async fn find_by_title(&self, title: &str) -> Result<Option<LearningOutcome>, AppError> {
        let filter = doc! { "title": title };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find learning outcome by title: {}", e),
            })
    }

    /// Insert new learning outcome
    pub async fn insert_outcome(
        &self,
        outcome: &LearningOutcome,
    ) -> Result<LearningOutcome, AppError> {
        // Unique index on subject_id + order combination
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

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted learning outcome id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Learning outcome not found after insert".to_string(),
            })
    }

    /// Get all learning outcomes (latest updated first)
    pub async fn get_all_outcomes(&self) -> Result<Vec<LearningOutcome>, AppError> {
        let pipeline = vec![doc! { "$sort": { "updated_at": -1 } }];

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

    /// Update learning outcome
    pub async fn update_outcome(
        &self,
        id: &IdType,
        update: UpdateLearningOutcome,
    ) -> Result<LearningOutcome, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        // Convert struct -> Document
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

        let update_query = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_query)
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
