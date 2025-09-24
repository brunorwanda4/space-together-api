use crate::domain::trade::{Trade, TradeWithSector, UpdateTrade};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
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
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find trade by id: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<Trade>, AppError> {
        let filter = doc! { "username": username };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find trade by username: {}", e),
            })
    }

    pub async fn insert_trade(&self, trade: &Trade) -> Result<Trade, AppError> {
        // Unique index for username
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Non-unique index for sector_id (to quickly filter by sector)
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
                message: format!("Failed to create sector_id index: {}", e),
            })?;

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
                message: "Failed to get inserted trade id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Trade not found after insert".to_string(),
            })
    }

    pub async fn get_all_trades(&self) -> Result<Vec<Trade>, AppError> {
        let pipeline = vec![doc! { "$sort": { "updated_at": -1 } }];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch trades: {}", e),
            })?;

        let mut trades = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate trades: {}", e),
        })? {
            let trade: Trade = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize trade: {}", e),
            })?;
            trades.push(trade);
        }

        Ok(trades)
    }

    pub async fn get_all_trades_with_sector(&self) -> Result<Vec<TradeWithSector>, AppError> {
        let pipeline = vec![
            doc! { "$sort": { "updated_at": -1 } },
            doc! {
                "$lookup": {
                    "from": "sectors",          // collection name
                    "localField": "sector_id",  // field in trades
                    "foreignField": "_id",      // field in sectors
                    "as": "sector"
                }
            },
            doc! {
                "$unwind": {
                    "path": "$sector",
                    "preserveNullAndEmptyArrays": true
                }
            },
        ];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch trades with sector: {}", e),
            })?;

        let mut trades = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate trades with sector: {}", e),
        })? {
            let trade: TradeWithSector = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize TradeWithSector: {}", e),
            })?;
            trades.push(trade);
        }

        Ok(trades)
    }

    pub async fn update_trade(&self, id: &IdType, update: &UpdateTrade) -> Result<Trade, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to convert update trade to document: {}", e),
        })?;

        // 🔥 Remove all `null` fields so they don’t overwrite existing values
        update_doc = update_doc
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
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

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
