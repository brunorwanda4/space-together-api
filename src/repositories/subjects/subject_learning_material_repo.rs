use crate::domain::subjects::subject_learning_material::{
    SubjectLearningMaterial, SubjectLearningMaterialRole, SubjectMaterialType,
    UpdateSubjectLearningMaterial,
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

pub struct SubjectLearningMaterialRepo {
    pub collection: Collection<SubjectLearningMaterial>,
}

impl SubjectLearningMaterialRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SubjectLearningMaterial>("subject_learning_materials"),
        }
    }

    /// Ensure indexes for better query performance and uniqueness
    pub async fn init_indexes(&self) -> Result<(), AppError> {
        // Index on reference_id
        let reference_index = IndexModel::builder()
            .keys(doc! { "reference_id": 1 })
            .build();

        // Compound index: role + reference_id (faster lookups per subject/topic type)
        let compound_index = IndexModel::builder()
            .keys(doc! { "role": 1, "reference_id": 1 })
            .options(IndexOptions::builder().build())
            .build();

        self.collection
            .create_indexes([reference_index, compound_index])
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create indexes: {}", e),
            })?;

        Ok(())
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

    /// Find by reference_id (regardless of role)
    pub async fn find_by_reference_id(
        &self,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(reference_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse reference_id: {}", e),
        })?;

        let filter = doc! { "reference_id": obj_id };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find learning materials by reference_id: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }

    /// Find by role + reference_id
    pub async fn find_by_role_and_reference(
        &self,
        role: &SubjectLearningMaterialRole,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(reference_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse reference_id: {}", e),
        })?;

        let filter = doc! {
            "role": bson::to_bson(role).map_err(|e| AppError {
                message: format!("Failed to serialize role: {}", e),
            })?,
            "reference_id": obj_id
        };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find materials by role and reference_id: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate learning materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }

    /// Find active materials by role + reference_id
    pub async fn find_active(
        &self,
        role: &SubjectLearningMaterialRole,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(reference_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse reference_id: {}", e),
        })?;

        let filter = doc! {
            "role": bson::to_bson(role).unwrap(),
            "reference_id": obj_id,
            "is_active": true
        };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find active materials: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate active materials: {}", e),
        })? {
            materials.push(result);
        }

        Ok(materials)
    }

    /// Find by material_type + role + reference_id
    pub async fn find_by_type_and_reference(
        &self,
        material_type: &SubjectMaterialType,
        role: &SubjectLearningMaterialRole,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, AppError> {
        let obj_id = ObjectId::parse_str(reference_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse reference_id: {}", e),
        })?;

        let filter = doc! {
            "material_type": bson::to_bson(material_type).unwrap(),
            "role": bson::to_bson(role).unwrap(),
            "reference_id": obj_id
        };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find by type and reference_id: {}", e),
        })?;

        let mut materials = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate type + reference_id materials: {}", e),
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
        self.init_indexes().await?;

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

    /// Update learning material
    pub async fn update_material(
        &self,
        id: &IdType,
        update: UpdateSubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(&update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        update_doc = update_doc
            .into_iter()
            .filter(|(k, v)| k != "_id" && !matches!(v, bson::Bson::Null))
            .collect();

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

    /// Delete material
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
}
