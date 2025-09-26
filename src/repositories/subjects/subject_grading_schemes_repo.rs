use crate::domain::subjects::subject_grading_schemes::{
    SubjectGradingScheme, UpdateSubjectGradingScheme,
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

pub struct SubjectGradingSchemesRepo {
    pub collection: Collection<SubjectGradingScheme>,
}

impl SubjectGradingSchemesRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SubjectGradingScheme>("subject_grading_schemes"),
        }
    }

    /// Find by id
    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<SubjectGradingScheme>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find grading scheme by id: {}", e),
            })
    }

    /// Find by main_subject_id
    pub async fn find_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<Option<SubjectGradingScheme>, AppError> {
        let obj_id = ObjectId::parse_str(main_subject_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse main_subject_id: {}", e),
        })?;

        let filter = doc! { "main_subject_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find grading scheme by main_subject_id: {}", e),
            })
    }

    /// Find by scheme_type
    pub async fn find_by_scheme_type(
        &self,
        scheme_type: &crate::domain::subjects::subject_grading_schemes::SubjectGradingType,
    ) -> Result<Vec<SubjectGradingScheme>, AppError> {
        let filter = doc! {
            "scheme_type": bson::to_bson(scheme_type).map_err(|e| AppError {
                message: format!("Failed to serialize scheme_type: {}", e),
            })?
        };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find grading schemes by scheme_type: {}", e),
        })?;

        let mut schemes = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate grading schemes: {}", e),
        })? {
            schemes.push(result);
        }

        Ok(schemes)
    }

    /// Insert new grading scheme
    pub async fn insert_scheme(
        &self,
        scheme: &SubjectGradingScheme,
    ) -> Result<SubjectGradingScheme, AppError> {
        // Unique index on main_subject_id + role combination
        let subject_role_index = IndexModel::builder()
            .keys(doc! { "main_subject_id": 1, "role": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(subject_role_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create unique subject_role index: {}", e),
            })?;

        let mut scheme_to_insert = scheme.clone();
        scheme_to_insert.id = None;
        scheme_to_insert.created_at = Some(Utc::now());
        scheme_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&scheme_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert grading scheme: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted grading scheme id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Grading scheme not found after insert".to_string(),
            })
    }

    /// Get all grading schemes (latest updated first)
    pub async fn get_all_schemes(&self) -> Result<Vec<SubjectGradingScheme>, AppError> {
        let pipeline = vec![doc! { "$sort": { "updated_at": -1 } }];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch grading schemes: {}", e),
            })?;

        let mut schemes = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate grading schemes: {}", e),
        })? {
            let scheme: SubjectGradingScheme =
                bson::from_document(result).map_err(|e| AppError {
                    message: format!("Failed to deserialize grading scheme: {}", e),
                })?;
            schemes.push(scheme);
        }

        Ok(schemes)
    }

    /// Update grading scheme
    pub async fn update_scheme(
        &self,
        id: &IdType,
        update: UpdateSubjectGradingScheme,
    ) -> Result<SubjectGradingScheme, AppError> {
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
                message: format!("Failed to update grading scheme: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Grading scheme not found after update".to_string(),
        })
    }

    /// Delete grading scheme
    pub async fn delete_scheme(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete grading scheme: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No grading scheme deleted; scheme may not exist".to_string(),
            });
        }

        Ok(())
    }

    /// Bulk find by main_subject_ids
    pub async fn find_by_main_subject_ids(
        &self,
        main_subject_ids: &[ObjectId],
    ) -> Result<Vec<SubjectGradingScheme>, AppError> {
        if main_subject_ids.is_empty() {
            return Ok(Vec::new());
        }

        let filter = doc! { "main_subject_id": { "$in": main_subject_ids } };
        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find grading schemes by main_subject_ids: {}", e),
        })?;

        let mut schemes = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate grading schemes: {}", e),
        })? {
            schemes.push(result);
        }

        Ok(schemes)
    }

    /// Find by main_subject_id and role
    pub async fn find_by_subject_and_role(
        &self,
        main_subject_id: &IdType,
        role: &crate::domain::subjects::subject_category::SubjectTypeFor,
    ) -> Result<Option<SubjectGradingScheme>, AppError> {
        let obj_id = ObjectId::parse_str(main_subject_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse main_subject_id: {}", e),
        })?;

        let filter = doc! {
            "main_subject_id": obj_id,
            "role": bson::to_bson(role).map_err(|e| AppError {
                message: format!("Failed to serialize role: {}", e),
            })?
        };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find grading scheme by subject and role: {}", e),
            })
    }
}
