use chrono::Utc;
use mongodb::{bson::{doc, Document}, Collection, Database};

use crate::{
    config::state::AppState,
    domain::{
        audit_log::AuditSeverity,
        auth_user::AuthUserDto,
        common_details::Paginated,
        learning_material::{LearningMaterial, LearningMaterialPartial, LearningMaterialWithRelations, MaterialType},
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::{CountDoc, IndexDef}},
    pipeline::learning_material_pipeline::learning_material_pipeline,
    repositories::base_repo::BaseRepository,
    services::{audit_log_service::AuditLogService, cloudinary_service::CloudinaryService},
    utils::mongo_utils::{build_search_filter, extract_valid_fields},
};

pub struct LearningMaterialService {
    pub collection: Collection<LearningMaterial>,
}

impl LearningMaterialService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<LearningMaterial>("learning_materials"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("school_id", false),
            IndexDef::single("class_id", false),
            IndexDef::single("subject_id", false),
            IndexDef::single("uploaded_by", false),
            IndexDef::single("material_type", false),
            IndexDef::single("is_published", false),
            IndexDef::single("created_at", false),
            IndexDef::single("deleted_at", false),
            IndexDef::compound(vec![("school_id", 1), ("subject_id", 1), ("created_at", -1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(
        &self,
        dto: LearningMaterial,
        file_bytes: Option<Vec<u8>>,
        file_name: Option<String>,
        user: &AuthUserDto,
        state: &AppState,
    ) -> Result<LearningMaterial, AppError> {
        self.ensure_indexes().await?;

        if dto.title.trim().is_empty() {
            return Err(AppError { message: "Title is required".into() });
        }

        let mut material = dto;

        if let (Some(bytes), Some(name)) = (file_bytes, file_name) {
            let school_id = material.school_id.ok_or(AppError { message: "School ID is required".into() })?;
            let subject_id = material.subject_id.ok_or(AppError { message: "Subject ID is required".into() })?;
            let folder = format!("space-together/{}/subjects/{}", school_id.to_hex(), subject_id.to_hex());

            let upload_result = CloudinaryService::upload_file(bytes, &name, &folder).await.map_err(|e| AppError { message: e })?;
            material.file_url = Some(upload_result.url);
            material.file_public_id = Some(upload_result.public_id);
        }

        if matches!(material.material_type, MaterialType::Video) && material.video_url.is_none() {
            return Err(AppError { message: "Video URL is required for VIDEO type".into() });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let created_material = repo.create::<LearningMaterial>(extract_valid_fields(material.to_document()?), None).await?;

        if let (Some(school_id), Some(entity_id)) = (created_material.school_id, created_material.id) {
            let audit_service = AuditLogService::new(&state.db.main_db());
            audit_service.log_event(school_id, user, "learning_material.create", "learning_material", entity_id, None, None, Some(AuditSeverity::INFO)).await.ok();
        }

        Ok(created_material)
    }

    pub async fn find_one(&self, id: Option<&IdType>, extra_match: Option<Document>) -> Result<LearningMaterial, AppError> {
        let mut filter = extra_match.unwrap_or_default();
        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<LearningMaterial>(filter, None).await?.ok_or(AppError { message: "Learning material not found".into() })
    }

    pub async fn get_all(&self, filter: Option<String>, limit: Option<i64>, skip: Option<i64>, extra_match: Document) -> Result<Paginated<LearningMaterial>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable = ["title", "description", "material_type", "_id", "school_id", "class_id", "subject_id", "uploaded_by"];
        let (data, total, total_pages, current_page) = repo.get_all::<LearningMaterial>(filter, &searchable, limit, skip, Some(extra_match)).await?;
        Ok(Paginated { data, total, total_pages, current_page })
    }

    pub async fn update(&self, id: &IdType, update: &LearningMaterialPartial, file_bytes: Option<Vec<u8>>, file_name: Option<String>, user: &AuthUserDto, state: &AppState) -> Result<LearningMaterial, AppError> {
        let existing = self.find_one(Some(id), None).await?;
        let mut update_data = update.clone();

        if let (Some(bytes), Some(name)) = (file_bytes, file_name) {
            if let Some(old_public_id) = existing.file_public_id.clone() {
                CloudinaryService::delete_file(&old_public_id).await.ok();
            }

            let school_id = existing.school_id.ok_or(AppError { message: "School ID is required".into() })?;
            let subject_id = existing.subject_id.ok_or(AppError { message: "Subject ID is required".into() })?;
            let folder = format!("space-together/{}/subjects/{}", school_id.to_hex(), subject_id.to_hex());

            let upload_result = CloudinaryService::upload_file(bytes, &name, &folder).await.map_err(|e| AppError { message: e })?;
            update_data.file_url = Some(Some(upload_result.url));
            update_data.file_public_id = Some(Some(upload_result.public_id));
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let updated = repo.update_one_and_fetch::<LearningMaterial>(id, extract_valid_fields(LearningMaterial::from_partial(update_data)?)).await?;

        if let (Some(school_id), Some(entity_id)) = (updated.school_id, updated.id) {
            let audit_service = AuditLogService::new(&state.db.main_db());
            audit_service.log_event(school_id, user, "learning_material.update", "learning_material", entity_id, None, None, Some(AuditSeverity::INFO)).await.ok();
        }

        Ok(updated)
    }

    pub async fn delete(&self, id: &IdType, user: &AuthUserDto, state: &AppState) -> Result<LearningMaterial, AppError> {
        let material = self.find_one(Some(id), None).await?;
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = doc! { "$set": { "deleted_at": mongodb::bson::to_bson(&Utc::now()).unwrap() } };
        repo.update_one_raw(id, update_doc).await?;

        if let (Some(school_id), Some(entity_id)) = (material.school_id, material.id) {
            let audit_service = AuditLogService::new(&state.db.main_db());
            audit_service.log_event(school_id, user, "learning_material.delete", "learning_material", entity_id, None, None, Some(AuditSeverity::WARNING)).await.ok();
        }

        Ok(material)
    }

    pub async fn get_all_with_relations(&self, filter: Option<String>, limit: Option<i64>, skip: Option<i64>, extra_match: Document) -> Result<Paginated<LearningMaterialWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let mut match_stage = extra_match;
        if let Some(f) = filter {
            let search = build_search_filter(Some(f), &["title", "description", "material_type", "_id", "school_id", "class_id", "subject_id"]);
            match_stage.extend(search);
        }
        let pipeline = learning_material_pipeline(match_stage);
        repo.aggregate_with_paginate::<LearningMaterialWithRelations>(pipeline, limit, skip).await
    }

    pub async fn find_one_with_relations(&self, id: Option<&IdType>, extra_match: Option<Document>) -> Result<LearningMaterialWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();
        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.aggregate_one::<LearningMaterialWithRelations>(learning_material_pipeline(match_stage), None).await?.ok_or(AppError { message: "Learning material not found".into() })
    }

    pub async fn count_materials(&self, filter: Option<String>, extra_match: Document) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable = ["title", "description", "material_type", "school_id", "class_id", "subject_id"];
        repo.count(filter, &searchable, Some(extra_match)).await
    }
}
