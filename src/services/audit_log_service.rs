use chrono::Utc;
use mongodb::{
    bson::{oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        audit_log::{AuditLog, AuditLogWithRelations, AuditSeverity, RequestMeta},
        auth_user::AuthUserDto,
        common_details::Paginated,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::audit_log_pipeline::audit_log_pipeline,
    repositories::base_repo::BaseRepository,
    utils::{mongo_utils::build_search_filter, object_id::parse_object_id_value},
};

pub struct AuditLogService {
    pub collection: Collection<AuditLog>,
}

impl AuditLogService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<AuditLog>("audit_logs"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("school_id", false),
            IndexDef::single("user_id", false),
            IndexDef::single("entity_type", false),
            IndexDef::single("entity_id", false),
            IndexDef::single("action", false),
            IndexDef::compound(vec![("created_at", -1)], false),
            IndexDef::compound(vec![("school_id", 1), ("created_at", -1)], false),
        ];

        let repo = BaseRepository::new(
            self.collection
                .clone()
                .clone_with_type::<mongodb::bson::Document>(),
        );

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    /// Log an audit event - NEVER fails the main transaction
    pub async fn log_event(
        &self,
        school_id: ObjectId,
        user: &AuthUserDto,
        action: &str,
        entity_type: &str,
        entity_id: ObjectId,
        metadata: Option<Document>,
        request_meta: Option<RequestMeta>,
        severity: Option<AuditSeverity>,
    ) -> Result<(), AppError> {
        // Silently fail if audit logging fails
        let result = self
            .log_event_internal(
                school_id,
                user,
                action,
                entity_type,
                entity_id,
                metadata,
                request_meta,
                severity,
            )
            .await;

        if let Err(e) = result {
            // Log error but don't propagate
            eprintln!("Audit log failed: {:?}", e);
        }

        Ok(())
    }

    async fn log_event_internal(
        &self,
        school_id: ObjectId,
        user: &AuthUserDto,
        action: &str,
        entity_type: &str,
        entity_id: ObjectId,
        metadata: Option<Document>,
        request_meta: Option<RequestMeta>,
        severity: Option<AuditSeverity>,
    ) -> Result<(), AppError> {
        self.ensure_indexes().await?;

        let user_id = parse_object_id_value(&user.id)?;

        let audit_log = AuditLog {
            id: None,
            school_id,
            user_id,
            user_role: user.role.clone().unwrap_or(crate::domain::common_details::UserRole::STUDENT),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            metadata,
            ip_address: request_meta.as_ref().and_then(|m| m.ip_address.clone()),
            user_agent: request_meta.as_ref().and_then(|m| m.user_agent.clone()),
            severity: severity.unwrap_or_default(),
            created_at: Utc::now(),
        };

        self.collection.insert_one(audit_log).await?;

        Ok(())
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<AuditLog, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<AuditLog>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Audit log not found".into(),
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
    ) -> Result<Paginated<AuditLog>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "action",
            "entity_type",
            "_id",
            "user_id",
            "school_id",
            "entity_id",
            "severity",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<AuditLog>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
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
    ) -> Result<Paginated<AuditLogWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &[
                    "action",
                    "entity_type",
                    "_id",
                    "user_id",
                    "school_id",
                    "entity_id",
                    "severity",
                ],
            );

            match_stage.extend(search);
        }

        let pipeline = audit_log_pipeline(match_stage);

        repo.aggregate_with_paginate::<AuditLogWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<AuditLogWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<AuditLogWithRelations>(audit_log_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Audit log not found".into(),
            })
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count_audit_logs(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "action",
            "entity_type",
            "user_id",
            "school_id",
            "entity_id",
            "severity",
        ];

        repo.count(filter, &searchable, extra_match).await
    }
}
