use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    config::state::AppState,
    domain::{
        common_details::Paginated,
        main_class::MainClass,
        trade::{Trade, TradePartial, TradeWithOthers},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::trade_pipeline::trade_pipeline,
    repositories::base_repo::BaseRepository,
    services::main_class_service::MainClassService,
    utils::mongo_utils::extract_valid_fields,
};

pub struct TradeService {
    pub collection: Collection<Trade>,
}

impl TradeService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Trade>("trades"),
        }
    }

    // =========================
    // INDEXES
    // =========================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", true),
            IndexDef::single("username", true),
            IndexDef::single("sector_id", false),
            IndexDef::single("trade_id", false),
            IndexDef::single("type", false),
            IndexDef::single("disable", false),
            IndexDef::compound(vec![("sector_id", 1), ("type", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: Trade, state: Option<&AppState>) -> Result<Trade, AppError> {
        self.ensure_indexes().await?;

        // unique name
        if let Ok(existing) = self.find_one(None, Some(doc! { "name": &dto.name })).await {
            return Err(AppError {
                message: format!("Trade name already exists: {}", existing.name),
            });
        }

        // unique username
        if let Ok(existing) = self
            .find_one(None, Some(doc! { "username": &dto.username }))
            .await
        {
            return Err(AppError {
                message: format!("Trade username already exists: {}", existing.username),
            });
        }

        let full_doc = bson::to_document(&dto).map_err(|e| AppError {
            message: format!("Failed to serialize trade: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let trade = repo
            .create::<Trade>(extract_valid_fields(full_doc), None)
            .await?;

        if let Some(state) = state {
            let trade_oid = trade.id.clone().ok_or_else(|| AppError {
                message: "Inserted trade has no id".to_string(),
            })?;

            let mut min = trade.class_min.clone();
            let mut max = trade.class_max.clone();
            if min > max {
                std::mem::swap(&mut min, &mut max);
            }

            if max < min {
                return Ok(trade);
            }

            let trade_type_name = trade.r#type.to_string();

            let mut main_classes_to_create = Vec::new();
            for level in min..=max {
                let name = format!("{} {} {}", trade_type_name, level, trade.clone().name);
                let username = format!(
                    "{}_{}_{}",
                    trade_type_name.to_lowercase(),
                    level,
                    trade.username.replace(' ', "_").to_lowercase()
                );

                main_classes_to_create.push(MainClass {
                    id: None,
                    name,
                    username,
                    trade_id: Some(trade_oid),
                    level: Some(level),
                    description: Some(format!("Auto-created for trade {}", trade.name)),
                    disable: Some(false),
                    created_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                });
            }

            let main_class_service = MainClassService::new(&state.db.main_db());

            let _ = if main_classes_to_create.is_empty() {
                vec![]
            } else {
                main_class_service
                    .create_many(main_classes_to_create)
                    .await?
            };
        }

        Ok(trade)
    }

    // =========================
    // FIND ONE (NO RELATIONS)
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Trade, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Trade>(filter, None).await?.ok_or(AppError {
            message: "Trade not found".into(),
        })
    }

    // =========================
    // GET ALL (NO RELATIONS)
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Trade>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "description", "type", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Trade>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // =========================
    // UPDATE
    // =========================
    pub async fn update(&self, id: &IdType, update: &TradePartial) -> Result<Trade, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        // name uniqueness
        if let Some(ref name) = update.name {
            if existing.name != *name {
                if let Ok(_) = self.find_one(None, Some(doc! { "name": name })).await {
                    return Err(AppError {
                        message: format!("Trade name already exists: {}", name),
                    });
                }
            }
        }

        // username uniqueness
        if let Some(ref username) = update.username {
            if existing.username != *username {
                if let Ok(_) = self
                    .find_one(None, Some(doc! { "username": username }))
                    .await
                {
                    return Err(AppError {
                        message: format!("Trade username already exists: {}", username),
                    });
                }
            }
        }

        let full_doc = bson::to_document(&update).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Trade>(id, extract_valid_fields(full_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Trade, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let trade = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(trade)
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "description", "type", "_id"];

        repo.count(filter, &searchable, extra_match).await
    }

    // =========================
    // GET ALL WITH RELATIONS
    // =========================
    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<TradeWithOthers>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let mut or_conditions = vec![
                doc! { "name": { "$regex": &f, "$options": "i" } },
                doc! { "username": { "$regex": &f, "$options": "i" } },
                doc! { "description": { "$regex": &f, "$options": "i" } },
                doc! { "type": { "$regex": &f, "$options": "i" } },
            ];

            if let Ok(oid) = ObjectId::parse_str(&f) {
                or_conditions.push(doc! { "_id": oid });
                or_conditions.push(doc! { "sector_id": oid });
                or_conditions.push(doc! { "trade_id": oid });
            }

            match_stage.insert("$or", or_conditions);
        }

        let pipeline = trade_pipeline(match_stage);

        repo.aggregate_with_paginate::<TradeWithOthers>(pipeline, limit, skip)
            .await
    }

    // =========================
    // FIND ONE WITH RELATIONS
    // =========================
    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<TradeWithOthers, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<TradeWithOthers>(trade_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Trade not found".into(),
            })
    }

    pub async fn find_by_ids(&self, ids: Vec<IdType>) -> Result<Vec<Trade>, AppError> {
        let object_ids: Vec<ObjectId> = ids
            .into_iter()
            .filter_map(|id| ObjectId::parse_str(id.as_string()).ok())
            .collect();

        if object_ids.is_empty() {
            return Ok(vec![]);
        }

        let filter = doc! { "_id": { "$in": object_ids } };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to fetch trades by ids: {}", e),
        })?;

        let mut trades = Vec::new();
        while let Some(trade) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate trades: {}", e),
        })? {
            trades.push(trade);
        }

        Ok(trades)
    }

    pub async fn get_trades_by_sector_ids(
        &self,
        sector_ids: &[ObjectId],
    ) -> Result<Vec<Trade>, AppError> {
        let filter = doc! { "sector_id": { "$in": sector_ids } };
        self.collection
            .find(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find trades by sector ids: {}", e),
            })?
            .try_collect()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to collect trades: {}", e),
            })
    }

    pub async fn get_trades_by_ids(&self, trade_ids: &[ObjectId]) -> Result<Vec<Trade>, AppError> {
        let filter = doc! { "_id": { "$in": trade_ids } };
        self.collection
            .find(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find trades by ids: {}", e),
            })?
            .try_collect()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to collect trades: {}", e),
            })
    }
}
