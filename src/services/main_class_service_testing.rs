use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        main_class::{MainClass, MainClassPartial, MainClassWithOthers},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::main_class_pipeline::main_class_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct MainClassService {
    pub collection: Collection<MainClass>,
}

impl MainClassService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<MainClass>("main_classes"),
        }
    }

    // =========================
    // INDEXES
    // =========================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", true),
            IndexDef::single("username", true),
            IndexDef::single("trade_id", false),
            IndexDef::single("level", false),
            IndexDef::single("disable", false),
            IndexDef::compound(vec![("trade_id", 1), ("level", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: MainClass) -> Result<MainClass, AppError> {
        self.ensure_indexes().await?;

        // unique name
        if let Ok(existing) = self.find_one(None, Some(doc! { "name": &dto.name })).await {
            return Err(AppError {
                message: format!("MainClass name already exists: {}", existing.name),
            });
        }

        // unique username
        if let Ok(existing) = self
            .find_one(None, Some(doc! { "username": &dto.username }))
            .await
        {
            return Err(AppError {
                message: format!("MainClass username already exists: {}", existing.username),
            });
        }

        let mut new_class = dto.clone();
        new_class.created_at = Some(chrono::Utc::now());

        let full_doc = bson::to_document(&new_class).map_err(|e| AppError {
            message: format!("Failed to serialize MainClass: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<MainClass>(extract_valid_fields(full_doc), None)
            .await
    }

    // =========================
    // FIND ONE (NO RELATIONS)
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<MainClass, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<MainClass>(filter, None)
            .await?
            .ok_or(AppError {
                message: "MainClass not found".into(),
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
    ) -> Result<Paginated<MainClass>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "description", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<MainClass>(filter, &searchable, limit, skip, extra_match)
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
    pub async fn update(
        &self,
        id: &IdType,
        update: &MainClassPartial,
    ) -> Result<MainClass, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        // name uniqueness
        if let Some(ref name) = update.name {
            if existing.name != *name {
                if let Ok(_) = self.find_one(None, Some(doc! { "name": name })).await {
                    return Err(AppError {
                        message: format!("MainClass name already exists: {}", name),
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
                        message: format!("MainClass username already exists: {}", username),
                    });
                }
            }
        }

        let mut update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        update_doc.insert("updated_at", chrono::Utc::now());

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<MainClass>(id, extract_valid_fields(update_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<MainClass, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let main_class = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(main_class)
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

        let searchable = ["name", "username", "description", "_id"];

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
    ) -> Result<Paginated<MainClassWithOthers>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let mut or_conditions = vec![
                doc! { "name": { "$regex": &f, "$options": "i" } },
                doc! { "username": { "$regex": &f, "$options": "i" } },
                doc! { "description": { "$regex": &f, "$options": "i" } },
            ];

            if let Ok(oid) = ObjectId::parse_str(&f) {
                or_conditions.push(doc! { "_id": oid });
                or_conditions.push(doc! { "trade_id": oid });
            }

            match_stage.insert("$or", or_conditions);
        }

        repo.aggregate_with_paginate::<MainClassWithOthers>(
            main_class_pipeline(match_stage),
            limit,
            skip,
        )
        .await
    }

    // =========================
    // FIND ONE WITH RELATIONS
    // =========================
    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<MainClassWithOthers, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<MainClassWithOthers>(main_class_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "MainClass not found".into(),
            })
    }
}
