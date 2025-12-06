use crate::{
    domain::common_details::Paginated,
    errors::AppError,
    helpers::repo_helpers::debug_deserialize_error,
    models::{id_model::IdType, mongo_model::IndexDef},
};
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, Document},
    options::IndexOptions,
    Collection, IndexModel,
};
use serde::de::DeserializeOwned;

pub struct BaseRepository {
    pub collection: Collection<Document>,
}

impl BaseRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        Self { collection }
    }

    /// Generic fetch-all function with filtering, pagination, and deserialization
    pub async fn get_all<T: DeserializeOwned>(
        &self,
        filter: Option<String>,
        searchable_fields: &[&str],
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<(Vec<T>, i64, i64, i64), AppError> {
        let mut pipeline = vec![];

        // ===== BUILD MATCH STAGE =====
        let mut match_stage = if let Some(f) = filter.clone() {
            let regex = doc! {
                "$regex": f,
                "$options": "i"
            };

            let or_conditions: Vec<Document> = searchable_fields
                .iter()
                .map(|field| doc! { *field: &regex })
                .collect();

            doc! { "$or": or_conditions }
        } else {
            doc! {}
        };

        // ===== MERGE EXTRA MATCH =====
        if let Some(extra) = extra_match {
            if !extra.is_empty() {
                if !match_stage.is_empty() {
                    match_stage = doc! { "$and": [match_stage, extra] };
                } else {
                    match_stage = extra;
                }
            }
        }

        // Add match to pipeline if not empty
        if !match_stage.is_empty() {
            pipeline.push(doc! { "$match": &match_stage });
        }

        // ===== COUNT TOTAL DOCUMENTS =====
        let mut count_pipeline = vec![];
        if !match_stage.is_empty() {
            count_pipeline.push(doc! { "$match": &match_stage });
        }
        count_pipeline.push(doc! { "$count": "total" });

        let total_cursor = self
            .collection
            .aggregate(count_pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count documents: {}", e),
            })?;

        let results: Vec<Document> = total_cursor.try_collect().await.unwrap_or_default();

        let total = results
            .first()
            .and_then(|doc| {
                doc.get_i32("total")
                    .ok()
                    .map(|v| v as i64)
                    .or_else(|| doc.get_i64("total").ok())
            })
            .unwrap_or(0);

        // ===== PAGINATION SETUP =====
        let limit_value = limit.unwrap_or(50).max(1); // Avoid 0 or negative limit
        let skip_value = skip.unwrap_or(0);

        pipeline.push(doc! { "$sort": { "updated_at": -1 } });
        if skip_value > 0 {
            pipeline.push(doc! { "$skip": skip_value });
        }
        pipeline.push(doc! { "$limit": limit_value });

        // ===== FETCH DOCUMENTS =====
        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch documents: {}", e),
            })?;

        let mut items = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate documents: {}", e),
        })? {
            let item: T = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize document: {}", e),
            })?;
            items.push(item);
        }

        // ===== COMPUTE PAGINATION INFO =====
        let current_page = skip_value / limit_value + 1;
        let total_pages = if total > 0 {
            ((total as f64) / (limit_value as f64)).ceil() as i64
        } else {
            1
        };

        Ok((items, total, total_pages, current_page))
    }

    /// Find a single document and deserialize it into type T
    pub async fn find_one<T: DeserializeOwned>(
        &self,
        filter: Document,
        extra_match: Option<Document>,
    ) -> Result<Option<T>, AppError> {
        // Merge filter + extra_match if provided
        let final_filter = if let Some(extra) = extra_match {
            if !extra.is_empty() {
                doc! { "$and": [filter, extra] }
            } else {
                filter
            }
        } else {
            filter
        };

        let result = self
            .collection
            .find_one(final_filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch document: {}", e),
            })?;

        // If no doc found → return Ok(None)
        let Some(doc) = result else {
            return Ok(None);
        };

        let item: T = bson::from_document(doc).map_err(|e| AppError {
            message: format!("Failed to deserialize document: {}", e),
        })?;

        Ok(Some(item))
    }

    /// Update a document and return the updated version
    pub async fn update_one_and_fetch<T: DeserializeOwned>(
        &self,
        id: &IdType,
        update_data: Document,
    ) -> Result<T, AppError> {
        let obj_id = IdType::to_object_id(id)?;

        if update_data.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        // Always update timestamp
        let mut update_doc = update_data.clone();
        update_doc.insert("updated_at", bson::to_bson(&chrono::Utc::now()).unwrap());

        // Perform update
        let result = self
            .collection
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update document: {}", e),
            })?;

        if result.matched_count == 0 {
            return Err(AppError {
                message: "Document not found".into(),
            });
        }

        // Fetch updated document
        let updated = self
            .collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated document: {}", e),
            })?
            .ok_or(AppError {
                message: "Failed to fetch updated document".into(),
            })?;

        let item: T = bson::from_document(updated).map_err(|e| AppError {
            message: format!("Failed to deserialize updated document: {}", e),
        })?;

        Ok(item)
    }

    /// Delete one document by ID
    pub async fn delete_one(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = IdType::to_object_id(id)?;

        let res = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete document: {}", e),
            })?;

        if res.deleted_count == 0 {
            Err(AppError {
                message: "Document not found".into(),
            })
        } else {
            Ok(())
        }
    }

    pub async fn create<T>(
        &self,
        mut doc: Document,
        unique_fields: Option<&[&str]>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
    {
        // ===== CREATE UNIQUE INDEXES =====
        if let Some(fields) = unique_fields {
            for field in fields {
                let index = IndexModel::builder()
                    .keys(doc! { *field: 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build();

                self.collection
                    .create_index(index)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to create unique index '{}': {}", field, e),
                    })?;
            }
        }

        // ===== PREPARE DOCUMENT =====
        doc.remove("_id");

        let now = chrono::Utc::now();
        doc.insert("created_at", bson::to_bson(&now).unwrap());
        doc.insert("updated_at", bson::to_bson(&now).unwrap());

        // ===== INSERT DOCUMENT =====
        let result = self
            .collection
            .insert_one(doc)
            .await
            .map_err(|e| AppError {
                message: format!("Insert failed: {}", e),
            })?;

        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| AppError {
            message: "Insert returned no object_id".into(),
        })?;

        // ===== FETCH INSERTED DOCUMENT =====
        let fetched = self
            .collection
            .find_one(doc! { "_id": inserted_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch inserted record: {}", e),
            })?
            .ok_or(AppError {
                message: "Inserted record not found".into(),
            })?;

        // ===== DESERIALIZE TO T =====
        let item: T = bson::from_document(fetched).map_err(|e| AppError {
            message: format!("Deserialize inserted record failed: {}", e),
        })?;

        Ok(item)
    }

    /// Create many documents fast and safely.
    /// - Ensures optional unique indexes
    /// - Auto adds timestamps
    /// - Inserts everything in one batch
    /// - Returns Vec<T> (fully deserialized)
    pub async fn create_many<T>(
        &self,
        docs: Vec<Document>,
        unique_fields: Option<&[&str]>,
    ) -> Result<Vec<T>, AppError>
    where
        T: DeserializeOwned,
    {
        // ===== ENSURE UNIQUE INDEXES (run once, fast) =====
        if let Some(fields) = unique_fields {
            for field in fields {
                let index = IndexModel::builder()
                    .keys(doc! { *field: 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build();

                self.collection
                    .create_index(index)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to create unique index '{}': {}", field, e),
                    })?;
            }
        }

        let insert_result = self
            .collection
            .insert_many(docs)
            .await
            .map_err(|e| AppError {
                message: format!("Insert many failed: {}", e),
            })?;

        // Collect all ObjectIds
        let ids: Vec<_> = insert_result
            .inserted_ids
            .values()
            .filter_map(|id| id.as_object_id())
            .collect();

        if ids.is_empty() {
            return Ok(vec![]);
        }

        // ===== FETCH ALL INSERTED DOCUMENTS =====
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": &ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch inserted documents: {}", e),
            })?;

        let mut items = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Iteration error: {}", e),
        })? {
            let item: T = bson::from_document(doc).map_err(|e| AppError {
                message: format!("Deserialize error: {}", e),
            })?;
            items.push(item);
        }

        Ok(items)
    }

    pub async fn ensure_indexes(&self, indexes: &[IndexDef]) -> Result<(), AppError> {
        for idx in indexes {
            let mut keys_doc = Document::new();
            for (field, order) in &idx.fields {
                keys_doc.insert(field, order);
            }

            // Build index
            let index = IndexModel::builder()
                .keys(keys_doc)
                .options(IndexOptions::builder().unique(idx.unique).build())
                .build();

            self.collection
                .create_index(index)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to create index on {:?}: {}", idx.fields, e),
                })?;
        }

        Ok(())
    }

    pub async fn aggregate_with_paginate<T>(
        &self,
        mut pipeline: Vec<Document>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<T>, AppError>
    where
        T: DeserializeOwned,
    {
        let limit_value = limit.unwrap_or(50).max(1);
        let skip_value = skip.unwrap_or(0);

        // Add pagination stages at end of pipeline
        pipeline.push(doc! { "$skip": skip_value });
        pipeline.push(doc! { "$limit": limit_value });

        let mut cursor = self
            .collection
            .aggregate(pipeline.clone())
            .await
            .map_err(|e| AppError {
                message: format!("Aggregation failed: {}", e),
            })?;

        let mut items: Vec<T> = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to read aggregate cursor: {}", e),
        })? {
            let item: T = bson::from_document(doc.clone()).map_err(|e| {
                // Convert BSON Document → JSON string
                let raw_json = serde_json::to_string_pretty(&doc)
                    .unwrap_or("<json encode failed>".to_string());

                // Debug the error
                debug_deserialize_error::<T>(&raw_json);

                AppError {
                    message: format!("Deserialize failed: {}", e),
                }
            })?;

            items.push(item);
        }

        let mut count_pipeline = pipeline.clone();
        count_pipeline
            .retain(|stage| !stage.contains_key("$skip") && !stage.contains_key("$limit"));

        count_pipeline.push(doc! { "$count": "total" });

        let mut count_cursor = self
            .collection
            .aggregate(count_pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Count aggregation failed: {}", e),
            })?;

        let total = if let Some(doc) = count_cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed reading count cursor: {}", e),
        })? {
            doc.get_i32("total")
                .ok()
                .map(|x| x as i64)
                .or_else(|| doc.get_i64("total").ok())
                .unwrap_or(0)
        } else {
            0
        };

        let current_page = skip_value / limit_value + 1;
        let total_pages = if total > 0 {
            ((total as f64) / (limit_value as f64)).ceil() as i64
        } else {
            1
        };

        Ok(Paginated {
            data: items,
            total,
            total_pages,
            current_page,
        })
    }
    /// Aggregate a pipeline and return a single deserialized document (with relationships).
    /// Aggregate a single document with lookup (relations) and deserialize into T.
    pub async fn aggregate_one<T>(
        &self,
        mut pipeline: Vec<Document>,
        extra_match: Option<Document>,
    ) -> Result<Option<T>, AppError>
    where
        T: DeserializeOwned,
    {
        // Optional extra $match merging
        if let Some(extra) = extra_match {
            if !extra.is_empty() {
                pipeline.insert(0, doc! { "$match": extra });
            }
        }

        // Force limit to 1 to ensure only one doc is returned
        pipeline.push(doc! { "$limit": 1 });

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Aggregation failed: {}", e),
            })?;

        // Return first doc if exists
        if let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed reading aggregate cursor: {}", e),
        })? {
            let item: T = bson::from_document(doc.clone()).map_err(|e| {
                let raw_json = serde_json::to_string_pretty(&doc)
                    .unwrap_or("<json encode failed>".to_string());

                // Debug the error
                debug_deserialize_error::<T>(&raw_json);

                AppError {
                    message: format!("Deserialize failed: {}", e),
                }
            })?;

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
