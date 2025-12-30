use mongodb::{
    bson::{self, doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        announcement::{Announcement, AnnouncementPartial, AnnouncementWithRelations},
        common_details::Paginated,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::announcement_pipeline::announcement_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct AnnouncementService {
    pub collection: Collection<Announcement>,
}

impl AnnouncementService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Announcement>("announcements"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(vec![("class_id", 1), ("created_at", -1)], false),
            IndexDef::compound(vec![("published.role", 1), ("created_at", -1)], false),
            IndexDef::single("created_at", false),
            IndexDef::single("published.id", false),
            IndexDef::single("mention.id", false),
        ];

        let repo = BaseRepository::new(
            self.collection
                .clone()
                .clone_with_type::<mongodb::bson::Document>(),
        );

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: Announcement) -> Result<Announcement, AppError> {
        self.ensure_indexes().await?;

        let partial = dto.to_partial();

        let full_doc = bson::to_document(&partial).map_err(|e| AppError {
            message: format!("Failed to serialize announcement: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<Announcement>(full_doc, None).await
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Announcement, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Announcement>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Announcement not found".into(),
            })
    }

    // =========================
    // GET ALL (PLAIN)
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Announcement>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["content", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Announcement>(filter, &searchable, limit, skip, extra_match)
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
        update: &AnnouncementPartial,
    ) -> Result<Announcement, AppError> {
        let full_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let update_doc = extract_valid_fields(full_doc);

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Announcement>(id, update_doc)
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Announcement, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let announcement = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;

        Ok(announcement)
    }

    // ======================================================
    // RELATIONSHIP QUERIES (PIPELINE)
    // ======================================================

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<AnnouncementWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            match_stage.insert(
                "$or",
                vec![doc! {
                    "content": { "$regex": &f, "$options": "i" }
                }],
            );
        }

        let pipeline = announcement_pipeline(match_stage);

        repo.aggregate_with_paginate::<AnnouncementWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<AnnouncementWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<AnnouncementWithRelations>(announcement_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Announcement not found".into(),
            })
    }

    pub async fn count_announcements(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Fields allowed for text / search filtering
        let searchable = [
            "content",
            "_id",
            "published.id",
            "published.role",
            "mention.id",
            "mention.role",
            "class_id",
        ];

        let total = base_repo.count(filter, &searchable, extra_match).await?;

        Ok(total)
    }
}
