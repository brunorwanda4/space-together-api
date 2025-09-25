use crate::domain::main_class::{
    MainClass, MainClassWithOthers, MainClassWithTrade, UpdateMainClass,
};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
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

    fn main_class_with_trade_pipeline(match_stage: Document) -> Vec<Document> {
        vec![
            doc! { "$match": match_stage },
            doc! {
                "$lookup": {
                    "from": "trades",
                    "localField": "trade_id",
                    "foreignField": "_id",
                    "as": "trade"
                }
            },
            doc! { "$unwind": { "path": "$trade", "preserveNullAndEmptyArrays": true } },
        ]
    }

    pub async fn get_all_with_trade(&self) -> Result<Vec<MainClassWithTrade>, AppError> {
        aggregate_many(&self.collection.clone().clone_with_type::<Document>(), {
            let mut pipeline = Self::main_class_with_trade_pipeline(doc! {});
            pipeline.insert(0, doc! { "$sort": { "updated_at": -1 } });
            pipeline
        })
        .await
    }

    fn main_class_with_others_pipeline(match_stage: Document) -> Vec<Document> {
        vec![
            doc! { "$match": match_stage },
            doc! {
                "$lookup": {
                    "from": "trades",
                    "localField": "trade_id",
                    "foreignField": "_id",
                    "as": "trade"
                }
            },
            doc! { "$unwind": { "path": "$trade", "preserveNullAndEmptyArrays": true } },
            // Include sector & parent_trade inside Trade
            doc! {
                "$lookup": {
                    "from": "sectors",
                    "localField": "trade.sector_id",
                    "foreignField": "_id",
                    "as": "trade.sector"
                }
            },
            doc! { "$unwind": { "path": "$trade.sector", "preserveNullAndEmptyArrays": true } },
            doc! {
                "$lookup": {
                    "from": "trades",
                    "localField": "trade.trade_id",
                    "foreignField": "_id",
                    "as": "trade.parent_trade"
                }
            },
            doc! {
                "$addFields": {
                    "trade.parent_trade": {
                        "$cond": {
                            "if": { "$gt": [{ "$size": "$trade.parent_trade" }, 0] },
                            "then": { "$arrayElemAt": ["$trade.parent_trade", 0] },
                            "else": null
                        }
                    }
                }
            },
            doc! {
                "$lookup": {
                    "from": "sectors",
                    "localField": "trade.parent_trade.sector_id",
                    "foreignField": "_id",
                    "as": "trade.parent_trade.sector"
                }
            },
            doc! { "$unwind": { "path": "$trade.parent_trade.sector", "preserveNullAndEmptyArrays": true } },
        ]
    }

    pub async fn find_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<Option<MainClassWithOthers>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            Self::main_class_with_others_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<Option<MainClassWithOthers>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            Self::main_class_with_others_pipeline(doc! { "username": username }),
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

    async fn ensure_indexes(&self) -> Result<(), AppError> {
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let trade_index = IndexModel::builder()
            .keys(doc! { "trade_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

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

        Ok(())
    }

    pub async fn get_all(&self) -> Result<Vec<MainClass>, AppError> {
        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            vec![doc! { "$sort": { "updated_at": -1 } }],
        )
        .await
    }

    pub async fn update_main_class(
        &self,
        id: &IdType,
        update: &UpdateMainClass,
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
}
