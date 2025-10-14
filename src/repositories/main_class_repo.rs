use crate::domain::main_class::{MainClass, MainClassWithOthers, MainClassWithTrade};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::pipeline::main_class_pipeline::{
    main_class_with_others_pipeline, main_class_with_trade_pipeline,
};
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct MainClassRepo {
    pub collection: Collection<MainClass>,
}

impl MainClassRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<MainClass>("main_classes"),
        }
    }

    pub async fn get_all_with_trade(&self) -> Result<Vec<MainClassWithTrade>, AppError> {
        aggregate_many(&self.collection.clone().clone_with_type::<Document>(), {
            let mut pipeline = main_class_with_trade_pipeline(doc! {});
            pipeline.insert(0, doc! { "$sort": { "updated_at": -1 } });
            pipeline
        })
        .await
    }

    pub async fn find_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<Option<MainClassWithOthers>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            main_class_with_others_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<Option<MainClassWithOthers>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            main_class_with_others_pipeline(doc! { "username": username }),
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<MainClass>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find main_class by id: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<MainClass>, AppError> {
        self.collection
            .find_one(doc! { "username": username })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find main_class by username: {}", e),
            })
    }

    pub async fn insert_main_class(&self, main_class: &MainClass) -> Result<MainClass, AppError> {
        self.ensure_indexes().await?;

        let mut to_insert = main_class.clone();
        to_insert.id = None;
        to_insert.created_at = Some(Utc::now());
        to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert main_class: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted main_class id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "MainClass not found after insert".to_string(),
            })
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        // Unique index on username
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Non-unique index on trade_id (for faster queries)
        let trade_index = IndexModel::builder()
            .keys(doc! { "trade_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        // ✅ Unique compound index: trade_id + level
        // This ensures you cannot have two main classes
        // with the same trade_id and same level
        let unique_trade_level_index = IndexModel::builder()
            .keys(doc! { "trade_id": 1, "level": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name("unique_trade_level_index".to_string())
                    .build(),
            )
            .build();

        // Create indexes
        self.collection
            .create_index(username_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create username index: {}", e),
            })?;

        self.collection
            .create_index(trade_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create trade_id index: {}", e),
            })?;

        self.collection
            .create_index(unique_trade_level_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create trade_id+level unique index: {}", e),
            })?;

        Ok(())
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        trade_id: Option<String>,
    ) -> Result<Vec<MainClass>, AppError> {
        let mut pipeline = vec![];

        // Optional text filter (matches name, username, or description)
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"
            };
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "username": &regex },
                        { "description": &regex }
                    ]
                }
            });
        }

        // Optional filter by trade_id
        if let Some(trade_id_str) = trade_id {
            if let Ok(oid) = ObjectId::parse_str(&trade_id_str) {
                pipeline.push(doc! {
                    "$match": { "trade_id": oid }
                });
            }
        }

        // Add a sort date (fallback between updated_at and created_at)
        pipeline.push(doc! {
            "$addFields": {
                "sort_date": { "$ifNull": [ "$updated_at", "$created_at" ] }
            }
        });

        // Sort newest first
        pipeline.push(doc! {
            "$sort": { "sort_date": -1 }
        });

        // Optional skip
        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }

        // Limit results (default 10)
        pipeline.push(doc! { "$limit": limit.unwrap_or(10) });

        // Run aggregation
        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch main classes: {}", e),
            })?;

        // Deserialize results
        let mut main_classes = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate main classes: {}", e),
        })? {
            let item: MainClass = mongodb::bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize main class: {}", e),
            })?;
            main_classes.push(item);
        }

        Ok(main_classes)
    }

    pub async fn update_main_class(
        &self,
        id: &IdType,
        update: &MainClass,
    ) -> Result<MainClass, AppError> {
        let obj_id = parse_object_id(id)?;
        let update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to convert update main_class to document: {}", e),
        })?;

        // remove null fields
        let mut update_doc: Document = update_doc
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
                message: format!("Failed to update main_class: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "MainClass not found after update".to_string(),
        })
    }

    pub async fn delete_main_class(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete main_class: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No main_class deleted; it may not exist".to_string(),
            });
        }
        Ok(())
    }

    pub async fn find_by_trade_id(
        &self,
        trade_id: &IdType,
    ) -> Result<Vec<MainClassWithTrade>, AppError> {
        let trade_obj_id = parse_object_id(trade_id)?;

        let filter = doc! { "trade_id": trade_obj_id };

        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            main_class_with_trade_pipeline(filter),
        )
        .await
    }

    pub async fn find_by_trade_and_level(
        &self,
        trade_id: &ObjectId,
        level: i32,
    ) -> Result<Option<MainClass>, AppError> {
        let filter = doc! { "trade_id": trade_id, "level": level };
        let result = self
            .collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
            })?;
        Ok(result)
    }
}
