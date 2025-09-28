use crate::domain::subjects::subject_progress_tracking_config::{
    SubjectProgressTrackingConfig, UpdateSubjectProgressTrackingConfig,
};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct SubjectProgressConfigsRepo {
    pub collection: Collection<SubjectProgressTrackingConfig>,
}

impl SubjectProgressConfigsRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SubjectProgressTrackingConfig>("subject_progress_configs"),
        }
    }

    /// Ensure unique index on reference_id
    pub async fn init_indexes(&self) -> Result<(), AppError> {
        let index_model = IndexModel::builder()
            .keys(doc! { "reference_id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(index_model)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create unique reference_id index: {}", e),
            })?;

        Ok(())
    }

    /// Find by reference_id
    pub async fn find_by_reference_id(
        &self,
        reference_id: &IdType,
    ) -> Result<Option<SubjectProgressTrackingConfig>, AppError> {
        let obj_id = ObjectId::parse_str(reference_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse reference_id: {}", e),
        })?;

        let filter = doc! { "reference_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find progress config by reference_id: {}", e),
            })
    }

    /// Find many by reference_ids
    pub async fn find_by_reference_ids(
        &self,
        reference_ids: &[ObjectId],
    ) -> Result<Vec<SubjectProgressTrackingConfig>, AppError> {
        if reference_ids.is_empty() {
            return Ok(Vec::new());
        }

        let filter = doc! { "reference_id": { "$in": reference_ids } };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find progress configs by reference_ids: {}", e),
        })?;

        let mut configs = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate progress configs: {}", e),
        })? {
            configs.push(result);
        }

        Ok(configs)
    }

    /// Find by id
    pub async fn find_by_id(
        &self,
        id: &IdType,
    ) -> Result<Option<SubjectProgressTrackingConfig>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find progress config by id: {}", e),
            })
    }

    /// Insert new progress config
    pub async fn insert_config(
        &self,
        config: &SubjectProgressTrackingConfig,
    ) -> Result<SubjectProgressTrackingConfig, AppError> {
        self.init_indexes().await?;

        let mut config_to_insert = config.clone();
        config_to_insert.id = None;
        config_to_insert.created_at = Some(Utc::now());
        config_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&config_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert progress config: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted progress config id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Progress config not found after insert".to_string(),
            })
    }

    /// Get all progress configs (latest updated first)
    pub async fn get_all_configs(&self) -> Result<Vec<SubjectProgressTrackingConfig>, AppError> {
        let pipeline = vec![doc! { "$sort": { "updated_at": -1 } }];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch progress configs: {}", e),
            })?;

        let mut configs = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate progress configs: {}", e),
        })? {
            let config: SubjectProgressTrackingConfig =
                bson::from_document(result).map_err(|e| AppError {
                    message: format!("Failed to deserialize progress config: {}", e),
                })?;
            configs.push(config);
        }

        Ok(configs)
    }

    /// Update progress config
    pub async fn update_config(
        &self,
        id: &IdType,
        update: UpdateSubjectProgressTrackingConfig,
    ) -> Result<SubjectProgressTrackingConfig, AppError> {
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
                message: format!("Failed to update progress config: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Progress config not found after update".to_string(),
        })
    }

    /// Delete progress config
    pub async fn delete_config(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete progress config: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No progress config deleted; config may not exist".to_string(),
            });
        }

        Ok(())
    }
}
