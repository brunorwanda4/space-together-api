use crate::domain::subjects::main_subject::{
    MainSubject, MainSubjectWithOthers, UpdateMainSubject,
};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::pipeline::main_subject_pipeline::main_subject_with_others_pipeline;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct MainSubjectRepo {
    pub collection: Collection<MainSubject>,
}

impl MainSubjectRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<MainSubject>("main_subjects"),
        }
    }

    /// Find by id with all related collections
    pub async fn find_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<Option<MainSubjectWithOthers>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            main_subject_with_others_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    /// Find by id
    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<MainSubject>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find main subject by id: {}", e),
            })
    }

    /// Find all main subjects that contain a specific main class ID
    pub async fn find_by_main_class_id(
        &self,
        main_class_id: &IdType,
    ) -> Result<Vec<MainSubject>, AppError> {
        let obj_id = ObjectId::parse_str(main_class_id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;
        let filter = doc! {
            "main_class_ids": obj_id
        };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to find main subjects by main class ID: {}", e),
        })?;

        let mut subjects = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate main subjects: {}", e),
        })? {
            subjects.push(result);
        }

        Ok(subjects)
    }

    /// Find by code
    pub async fn find_by_code(&self, code: &str) -> Result<Option<MainSubject>, AppError> {
        let filter = doc! { "code": code };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find main subject by code: {}", e),
            })
    }

    /// Get all main subjects with all related collections
    pub async fn get_all_subjects_with_others(
        &self,
    ) -> Result<Vec<MainSubjectWithOthers>, AppError> {
        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            main_subject_with_others_pipeline(doc! {}),
        )
        .await
    }

    /// Insert new main subject
    pub async fn insert_subject(&self, subject: &MainSubject) -> Result<MainSubject, AppError> {
        // Unique index on code
        let code_index = IndexModel::builder()
            .keys(doc! { "code": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(code_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create unique code index: {}", e),
            })?;

        let mut subject_to_insert = subject.clone();
        subject_to_insert.id = None;
        subject_to_insert.created_at = Some(Utc::now());
        subject_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&subject_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert main subject: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted main subject id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Main subject not found after insert".to_string(),
            })
    }

    /// Get all main subjects (latest updated first)
    pub async fn get_all_subjects(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        is_active: Option<bool>, // optional filter for active/inactive subjects
    ) -> Result<Vec<MainSubject>, AppError> {
        let mut pipeline = vec![];

        // 🔍 Text filter: name, code, description, level
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"
            };
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "code": &regex },
                        { "description": &regex },
                        { "level": &regex }
                    ]
                }
            });
        }

        // ⚙️ Optional filter by active state
        if let Some(active) = is_active {
            pipeline.push(doc! {
                "$match": { "is_active": active }
            });
        }

        // 🧮 Add computed field for sorting
        pipeline.push(doc! {
            "$addFields": {
                "sort_date": { "$ifNull": [ "$updated_at", "$created_at" ] }
            }
        });

        // 📅 Sort newest first
        pipeline.push(doc! {
            "$sort": { "sort_date": -1 }
        });

        // ⏩ Optional skip
        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }

        // 🔢 Limit results (default 10)
        pipeline.push(doc! { "$limit": limit.unwrap_or(10) });

        // 🧩 (Optional future addition) Lookup related main classes
        // Uncomment later if needed
        /*
        pipeline.push(doc! {
            "$lookup": {
                "from": "main_classes",
                "localField": "main_class_ids",
                "foreignField": "_id",
                "as": "main_classes"
            }
        });
        */

        // 📥 Execute aggregation
        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch main subjects: {}", e),
            })?;

        // 🧾 Deserialize results
        let mut subjects = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate main subjects: {}", e),
        })? {
            let subject: MainSubject =
                mongodb::bson::from_document(result).map_err(|e| AppError {
                    message: format!("Failed to deserialize main subject: {}", e),
                })?;
            subjects.push(subject);
        }

        Ok(subjects)
    }

    /// Update main subject
    pub async fn update_subject(
        &self,
        id: &IdType,
        update: UpdateMainSubject,
    ) -> Result<MainSubject, AppError> {
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
                message: format!("Failed to update main subject: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Main subject not found after update".to_string(),
        })
    }

    /// Delete main subject
    pub async fn delete_subject(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete main subject: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No main subject deleted; subject may not exist".to_string(),
            });
        }

        Ok(())
    }
}
