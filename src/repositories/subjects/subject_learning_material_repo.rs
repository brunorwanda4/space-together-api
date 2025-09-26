use crate::domain::subjects::subject_learning_material::{
    SubjectLearningMaterial, UpdateSubjectLearningMaterial,
};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    Collection, Database, IndexModel,
};

pub struct SubjectLearningMaterialRepo {
    pub collection: Collection<SubjectLearningMaterial>,
}

impl SubjectLearningMaterialRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SubjectLearningMaterial>("subject_learning_materials"),
        }
    }

    /// Find by id
    pub async fn find_by_id(
        &self,
        id: &IdType,
    ) -> Result<Option<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find learning material by id: {}", e),
            })
    }

    /// Find by subject_id
    pub async fn find_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(subject_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse subject_id: {}", e),
        })?;

        let filter = doc! { "subject_id": obj_id };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find learning materials by subject_id: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }

    /// Find active materials by subject_id
    pub async fn find_active_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(subject_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse subject_id: {}", e),
        })?;

        let filter = doc! {
            "subject_id": obj_id,
            "is_active": true
        };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!(
                "Failed to find active learning materials by subject_id: {}",
                e
            ),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }

    /// Find by material_type and subject_id
    pub async fn find_by_type_and_subject(
        &self,
        material_type: &crate::domain::subjects::subject_learning_material::SubjectMaterialType,
        subject_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(subject_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse subject_id: {}", e),
        })?;

        let filter = doc! {
            "subject_id": obj_id,
            "material_type": bson::to_bson(material_type).map_err(|e| AppError {
                message: format!("Failed to serialize material_type: {}", e),
            })?
        };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!(
                "Failed to find learning materials by type and subject: {}",
                e
            ),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }

    /// Insert new learning material
    pub async fn insert_material(
        &self,
        material: &SubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, AppError> {
        // Index for better performance
        let subject_id_index = IndexModel::builder().keys(doc! { "subject_id": 1 }).build();

        let active_materials_index = IndexModel::builder()
            .keys(doc! { "subject_id": 1, "is_active": 1 })
            .build();

        self.collection
            .create_index(subject_id_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create subject_id index: {}", e),
            })?;

        self.collection
            .create_index(active_materials_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create active materials index: {}", e),
            })?;

        let mut material_to_insert = material.clone();
        material_to_insert.id = None;
        material_to_insert.created_at = Some(Utc::now());
        material_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&material_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert learning material: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted learning material id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Learning material not found after insert".to_string(),
            })
    }

    /// Get all learning materials (latest updated first)
    pub async fn get_all_materials(&self) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let mut cursor = self.collection.find(doc! {}).await.map_err(|e| AppError {
            message: format!("Failed to fetch learning materials: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        // Sort by updated_at descending (latest updated first)
        materials.sort_by(|a, b| match (&b.updated_at, &a.updated_at) {
            (Some(b_dt), Some(a_dt)) => b_dt.cmp(a_dt),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        Ok(materials)
    }

    /// Update learning material
    pub async fn update_material(
        &self,
        id: &IdType,
        update: UpdateSubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, AppError> {
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
                message: format!("Failed to update learning material: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Learning material not found after update".to_string(),
        })
    }

    /// Delete learning material
    pub async fn delete_material(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete learning material: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No learning material deleted; material may not exist".to_string(),
            });
        }

        Ok(())
    }

    /// Bulk find by subject_ids
    pub async fn find_by_subject_ids(
        &self,
        subject_ids: &[ObjectId],
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        if subject_ids.is_empty() {
            return Ok(Vec::new());
        }

        let filter = doc! { "subject_id": { "$in": subject_ids } };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find learning materials by subject_ids: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }
}
