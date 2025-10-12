use crate::domain::trade::{Trade, TradeWithOthers, UpdateTrade};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::pipeline::trade_pipeline::trade_with_others_pipeline;
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct TradeRepo {
    pub collection: Collection<Trade>,
}

impl TradeRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Trade>("trades"),
        }
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<Trade>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find trade by id: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<Trade>, AppError> {
        self.collection
            .find_one(doc! { "username": username })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find trade by username: {}", e),
            })
    }

    pub async fn find_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<Option<TradeWithOthers>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(), // convert to Document
            trade_with_others_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<Option<TradeWithOthers>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            trade_with_others_pipeline(doc! { "username": username }),
        )
        .await
    }

    pub async fn insert_trade(&self, trade: &Trade) -> Result<Trade, AppError> {
        self.ensure_indexes().await?;

        let mut trade_to_insert = trade.clone();
        trade_to_insert.id = None;
        trade_to_insert.created_at = Some(Utc::now());
        trade_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&trade_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert trade: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted trade id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Trade not found after insert".to_string(),
            })
    }

    async fn ensure_indexes(&self) -> Result<(), AppError> {
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let sector_index = IndexModel::builder()
            .keys(doc! { "sector_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        self.collection
            .create_index(username_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create username index: {}", e),
            })?;

        self.collection
            .create_index(sector_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create sector index: {}", e),
            })?;

        Ok(())
    }

    pub async fn get_all_trades(&self) -> Result<Vec<Trade>, AppError> {
        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            vec![doc! { "$sort": { "updated_at": -1 } }],
        )
        .await
    }

    pub async fn get_all_trades_with_others(&self) -> Result<Vec<TradeWithOthers>, AppError> {
        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            trade_with_others_pipeline(doc! { "$expr": { "$ne": ["$_id", null] }}),
        )
        .await
    }

    pub async fn update_trade(&self, id: &IdType, update: &UpdateTrade) -> Result<Trade, AppError> {
        let obj_id = parse_object_id(id)?;
        let update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to convert update trade to document: {}", e),
        })?;

        // 🔥 Remove all `null` fields
        let mut update_doc: Document = update_doc
            .into_iter()
            .filter(|(_, v)| !matches!(v, bson::Bson::Null))
            .collect();

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());
        update_doc.remove("_id"); // prevent overwriting id

        let update_doc = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update trade: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Trade not found after update".to_string(),
        })
    }

    pub async fn delete_trade(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete trade: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No trade deleted; trade may not exist".to_string(),
            });
        }
        Ok(())
    }
}
