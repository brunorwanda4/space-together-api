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
    models::id_model::IdType,
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

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: Announcement) -> Result<Announcement, AppError> {
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
    pub async fn find_one_by_id(&self, id: &IdType) -> Result<Announcement, AppError> {
        let obj = IdType::to_object_id(id)?;
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let result = repo
            .find_one::<Announcement>(doc! { "_id": obj }, None)
            .await?;

        result.ok_or(AppError {
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
        let existing = self.find_one_by_id(id).await?;
        repo.delete_one(id).await?;
        Ok(existing)
    }

    // ======================================================
    // RELATIONSHIP QUERIES (PIPELINE)
    // ======================================================

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        class_id: Option<&IdType>,
    ) -> Result<Paginated<AnnouncementWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = doc! {};

        if let Some(f) = filter {
            match_stage.insert(
                "$or",
                vec![doc! { "content": { "$regex": &f, "$options": "i" } }],
            );
        }

        if let Some(cid) = class_id {
            let obj = IdType::to_object_id(cid)?;
            match_stage.insert("class_id", obj);
        }

        let pipeline = announcement_pipeline(match_stage);

        repo.aggregate_with_paginate::<AnnouncementWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
    ) -> Result<AnnouncementWithRelations, AppError> {
        let obj_id = IdType::to_object_id(id)?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let pipeline = announcement_pipeline(doc! {
            "_id": obj_id
        });

        let result = repo
            .aggregate_one::<AnnouncementWithRelations>(pipeline, None)
            .await?;

        result.ok_or(AppError {
            message: "Announcement not found".into(),
        })
    }
}
