use crate::domain::subjects::subject_topic::{SubjectTopic, UpdateSubjectTopic};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    Collection, Database,
};

pub struct SubjectTopicRepo {
    pub collection: Collection<SubjectTopic>,
}

impl SubjectTopicRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SubjectTopic>("subject_topics"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        // Unique index: one learning outcome can't have two topics with the same order
        let index = IndexModel::builder()
            .keys(doc! { "learning_outcome_id": 1, "learning_outcome_id" : 1, "order": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Optionally, a normal (non-unique) index on parent_topic_id for faster lookups
        let parent_index = IndexModel::builder()
            .keys(doc! { "parent_topic_id": 1, "learning_outcome_id" : 1, "order" : 1 })
            .options(IndexOptions::builder().build())
            .build();

        self.collection
            .create_indexes(vec![index, parent_index])
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create indexes: {}", e),
            })?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<SubjectTopic>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find topic by id: {}", e),
            })
    }

    pub async fn exists_order_in_learning_outcome(
        &self,
        learning_outcome_id: &IdType,
        order: f32,
    ) -> Result<bool, AppError> {
        let lo_obj_id = learning_outcome_id.to_object_id()?; // helper function if you have it
        let filter = doc! {
            "learning_outcome_id": lo_obj_id,
            "order": order
        };

        let existing = self
            .collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to query topics: {}", e),
            })?;

        Ok(existing.is_some())
    }

    /// Check if a topic with the same order exists under the same parent topic
    pub async fn exists_order_in_parent_topic(
        &self,
        parent_topic_id: &IdType,
        order: f32,
    ) -> Result<bool, AppError> {
        let parent_obj_id = parent_topic_id.to_object_id()?;
        let filter = doc! {
            "parent_topic_id": parent_obj_id,
            "order": order
        };

        let existing = self
            .collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to query topics: {}", e),
            })?;

        Ok(existing.is_some())
    }

    /// Fetch all topics filtered by learning_outcome_id and parent_topic_id, sorted by order ascending.
    pub async fn get_topics_by_learning_outcome_and_parent(
        &self,
        learning_outcome_id: &IdType,
        parent_topic_id: Option<&IdType>,
    ) -> Result<Vec<SubjectTopic>, AppError> {
        let lo_obj_id = IdType::to_object_id(learning_outcome_id)?;

        // Base filter: learning_outcome_id must match
        let mut filter = doc! { "learning_outcome_id": lo_obj_id };

        // Add parent_topic_id condition (null means top-level topics)
        if let Some(parent_id) = parent_topic_id {
            let parent_obj_id =
                ObjectId::parse_str(parent_id.as_string()).map_err(|e| AppError {
                    message: format!("Failed to parse parent_topic_id: {}", e),
                })?;
            filter.insert("parent_topic_id", parent_obj_id);
        } else {
            filter.insert("parent_topic_id", bson::Bson::Null);
        }

        // Sorting by "order"

        let mut cursor = self
            .collection
            .find(filter)
            .sort(doc! { "order": 1 })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch topics: {}", e),
            })?;

        let mut topics = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate topics: {}", e),
        })? {
            topics.push(result);
        }

        Ok(topics)
    }

    pub async fn insert_topic(&self, topic: &SubjectTopic) -> Result<SubjectTopic, AppError> {
        self.ensure_indexes().await?;

        let mut topic_to_insert = topic.clone();
        topic_to_insert.id = None;
        topic_to_insert.created_at = Some(Utc::now());
        topic_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&topic_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert topic: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted topic id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Topic not found after insert".to_string(),
            })
    }

    pub async fn get_all_topics(&self) -> Result<Vec<SubjectTopic>, AppError> {
        let pipeline = vec![doc! { "$sort": { "order": 1 } }]; // sort by order

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch topics: {}", e),
            })?;

        let mut topics = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate topics: {}", e),
        })? {
            let topic: SubjectTopic = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize topic: {}", e),
            })?;
            topics.push(topic);
        }

        Ok(topics)
    }

    pub async fn update_topic(
        &self,
        id: &IdType,
        update: &UpdateSubjectTopic,
    ) -> Result<SubjectTopic, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to convert update topic to document: {}", e),
        })?;

        // Remove nulls
        update_doc = update_doc
            .into_iter()
            .filter(|(_, v)| !matches!(v, bson::Bson::Null))
            .collect();

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());
        update_doc.remove("_id");

        let update_doc = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update topic: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Topic not found after update".to_string(),
        })
    }

    pub async fn delete_topic(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete topic: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No topic deleted; topic may not exist".to_string(),
            });
        }

        Ok(())
    }
}
