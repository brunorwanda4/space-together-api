use crate::domain::subjects::subject_topic::{SubjectTopic, UpdateSubjectTopic};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
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
            collection: db.collection::<SubjectTopic>("topics"),
        }
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

    pub async fn insert_topic(&self, topic: &SubjectTopic) -> Result<SubjectTopic, AppError> {
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
