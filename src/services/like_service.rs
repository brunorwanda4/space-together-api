use mongodb::{
    bson::{self, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        like::{Like, LikePartial, LikeWithRelations},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::like_pipeline::like_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct LikeService {
    pub collection: Collection<Like>,
}

impl LikeService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Like>("likes"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            // One reaction per user per target
            IndexDef::compound(
                vec![("actor.id", 1), ("target_id", 1), ("target_type", 1)],
                true,
            ),
            IndexDef::single("target_id", false),
            IndexDef::single("like", false),
            IndexDef::single("created_at", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(&self, dto: Like) -> Result<Like, AppError> {
        self.ensure_indexes().await?;

        let partial = dto.to_partial();
        let full_doc = bson::to_document(&partial).map_err(|e| AppError {
            message: format!("Failed to serialize like: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<Like>(full_doc, None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Like, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<Like>(filter, None).await?.ok_or(AppError {
            message: "Like not found".into(),
        })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Like>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable = ["_id", "target_id", "actor.id", "like"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Like>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update(&self, id: &IdType, update: &LikePartial) -> Result<Like, AppError> {
        let full_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let update_doc = extract_valid_fields(full_doc);
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Like>(id, update_doc).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Like, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let like = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(like)
    }

    pub async fn get_all_with_relations(
        &self,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<LikeWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let match_stage = extra_match.unwrap_or_default();

        repo.aggregate_with_paginate::<LikeWithRelations>(like_pipeline(match_stage), limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<LikeWithRelations>, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<LikeWithRelations>(like_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "like not found".into(),
            })
    }

    pub async fn count_likes(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable = ["target_id", "actor.id", "like"];
        let total = repo.count(filter, &searchable, extra_match).await?;
        Ok(total)
    }
}
