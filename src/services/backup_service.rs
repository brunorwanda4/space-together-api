use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use std::path::PathBuf;
use std::process::Command;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        backup::{BackupStatus, BackupType, SchoolBackup, SchoolBackupWithRelations},
        common_details::Paginated,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::backup_pipeline::backup_pipeline,
    repositories::base_repo::BaseRepository,
    services::audit_log_service::AuditLogService,
    utils::mongo_utils::build_search_filter,
};

pub struct BackupService {
    pub collection: Collection<SchoolBackup>,
}

impl BackupService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SchoolBackup>("school_backups"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("school_id", false),
            IndexDef::single("status", false),
            IndexDef::single("backup_type", false),
            IndexDef::single("created_by", false),
            IndexDef::compound(vec![("school_id", 1), ("created_at", -1)], false),
            IndexDef::compound(vec![("school_id", 1), ("status", 1)], false),
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
    // CREATE MANUAL BACKUP
    // =========================
    pub async fn create_manual_backup(
        &self,
        school_id: ObjectId,
        user: &AuthUserDto,
        state: &AppState,
    ) -> Result<SchoolBackup, AppError> {
        self.ensure_indexes().await?;

        let backup_name = format!(
            "manual_backup_{}_{}",
            school_id.to_hex(),
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        let file_path = format!("backups/{}.archive", backup_name);

        // Create initial backup record
        let mut backup = SchoolBackup {
            id: None,
            school_id: Some(school_id),
            backup_name: backup_name.clone(),
            backup_type: BackupType::Manual,
            file_path: file_path.clone(),
            size_bytes: 0,
            status: BackupStatus::InProgress,
            created_by: Some(ObjectId::parse_str(&user.id).map_err(|_| AppError {
                message: "Invalid user ID".to_string(),
            })?),
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let created_backup = repo
            .create::<SchoolBackup>(backup.to_document()?, None)
            .await?;

        let backup_id = created_backup.id.ok_or(AppError {
            message: "Backup ID not found".to_string(),
        })?;

        // Log audit event
        let audit_service = AuditLogService::new(&state.db.main_db());
        audit_service
            .log_event(
                school_id,
                user,
                "backup.manual.create",
                "backup",
                backup_id,
                Some(doc! {
                    "backup_name": &backup_name,
                    "file_path": &file_path,
                }),
                None,
                None,
            )
            .await
            .ok();

        // Spawn background task for actual backup
        let backup_id_clone = backup_id;
        let school_id_clone = school_id;
        let file_path_clone = file_path.clone();
        let collection_clone = self.collection.clone();
        let db_name = format!("school_{}", school_id.to_hex());

        actix_rt::spawn(async move {
            let result = Self::perform_backup(&db_name, &file_path_clone).await;

            let mut update_doc = doc! {};

            match result {
                Ok(size) => {
                    update_doc.insert("status", mongodb::bson::to_bson(&BackupStatus::Completed).unwrap());
                    update_doc.insert("size_bytes", size);
                    update_doc.insert("completed_at", mongodb::bson::to_bson(&Utc::now()).unwrap());
                }
                Err(e) => {
                    update_doc.insert("status", mongodb::bson::to_bson(&BackupStatus::Failed).unwrap());
                    update_doc.insert("error_message", e.message);
                    update_doc.insert("completed_at", mongodb::bson::to_bson(&Utc::now()).unwrap());
                }
            }

            collection_clone
                .update_one(
                    doc! { "_id": backup_id_clone },
                    doc! { "$set": update_doc },
                )
                .await
                .ok();
        });

        Ok(created_backup)
    }

    // =========================
    // PERFORM BACKUP (Internal)
    // =========================
    async fn perform_backup(db_name: &str, file_path: &str) -> Result<i64, AppError> {
        // Create backup directory if it doesn't exist
        let backup_dir = PathBuf::from("backups");
        std::fs::create_dir_all(&backup_dir).map_err(|e| AppError {
            message: format!("Failed to create backup directory: {}", e),
        })?;

        // Get MongoDB connection string from environment
        let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        // Execute mongodump command
        let output = Command::new("mongodump")
            .arg("--uri")
            .arg(&mongo_uri)
            .arg("--db")
            .arg(db_name)
            .arg("--archive")
            .arg(file_path)
            .arg("--gzip")
            .output()
            .map_err(|e| AppError {
                message: format!("Failed to execute mongodump: {}", e),
            })?;

        if !output.status.success() {
            return Err(AppError {
                message: format!(
                    "Backup failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        // Get file size
        let metadata = std::fs::metadata(file_path).map_err(|e| AppError {
            message: format!("Failed to get backup file size: {}", e),
        })?;

        Ok(metadata.len() as i64)
    }

    // =========================
    // RESTORE BACKUP
    // =========================
    pub async fn restore_backup(
        &self,
        backup_id: &IdType,
        user: &AuthUserDto,
        state: &AppState,
    ) -> Result<SchoolBackup, AppError> {
        // Find backup
        let backup = self.find_one(Some(backup_id), None).await?;

        // Validate backup status
        if !matches!(backup.status, BackupStatus::Completed) {
            return Err(AppError {
                message: "Cannot restore: backup is not completed".to_string(),
            });
        }

        // Validate backup file exists
        if !std::path::Path::new(&backup.file_path).exists() {
            return Err(AppError {
                message: "Backup file not found".to_string(),
            });
        }

        let school_id = backup.school_id.ok_or(AppError {
            message: "Backup has no school_id".to_string(),
        })?;

        // Check for ongoing restore (simple flag check)
        let ongoing_restore = self
            .collection
            .find_one(doc! {
                "school_id": school_id,
                "status": "RESTORING"
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to check restore status: {}", e),
            })?;

        if ongoing_restore.is_some() {
            return Err(AppError {
                message: "Another restore is already in progress for this school".to_string(),
            });
        }

        // Create restore record
        let restore_record = SchoolBackup {
            id: None,
            school_id: Some(school_id),
            backup_name: format!("restore_{}", backup.backup_name),
            backup_type: BackupType::Manual,
            file_path: backup.file_path.clone(),
            size_bytes: backup.size_bytes,
            status: BackupStatus::InProgress,
            created_by: Some(ObjectId::parse_str(&user.id).map_err(|_| AppError {
                message: "Invalid user ID".to_string(),
            })?),
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let restore_doc = repo
            .create::<SchoolBackup>(restore_record.to_document()?, None)
            .await?;

        // Log audit event
        let audit_service = AuditLogService::new(&state.db.main_db());
        audit_service
            .log_event(
                school_id,
                user,
                "backup.restore",
                "backup",
                backup_id.to_object_id()?,
                Some(doc! {
                    "backup_name": &backup.backup_name,
                    "file_path": &backup.file_path,
                }),
                None,
                None,
            )
            .await
            .ok();

        // Perform restore in background
        let file_path = backup.file_path.clone();
        let db_name = format!("school_{}", school_id.to_hex());
        let restore_id = restore_doc.id.ok_or(AppError {
            message: "Restore ID not found".to_string(),
        })?;
        let collection_clone = self.collection.clone();

        actix_rt::spawn(async move {
            let result = Self::perform_restore(&db_name, &file_path).await;

            let mut update_doc = doc! {};

            match result {
                Ok(_) => {
                    update_doc.insert("status", mongodb::bson::to_bson(&BackupStatus::Completed).unwrap());
                    update_doc.insert("completed_at", mongodb::bson::to_bson(&Utc::now()).unwrap());
                }
                Err(e) => {
                    update_doc.insert("status", mongodb::bson::to_bson(&BackupStatus::Failed).unwrap());
                    update_doc.insert("error_message", e.message);
                    update_doc.insert("completed_at", mongodb::bson::to_bson(&Utc::now()).unwrap());
                }
            }

            collection_clone
                .update_one(doc! { "_id": restore_id }, doc! { "$set": update_doc })
                .await
                .ok();
        });

        Ok(restore_doc)
    }

    // =========================
    // PERFORM RESTORE (Internal)
    // =========================
    async fn perform_restore(db_name: &str, file_path: &str) -> Result<(), AppError> {
        let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        // Execute mongorestore command
        let output = Command::new("mongorestore")
            .arg("--uri")
            .arg(&mongo_uri)
            .arg("--db")
            .arg(db_name)
            .arg("--archive")
            .arg(file_path)
            .arg("--gzip")
            .arg("--drop") // Drop existing collections before restore
            .output()
            .map_err(|e| AppError {
                message: format!("Failed to execute mongorestore: {}", e),
            })?;

        if !output.status.success() {
            return Err(AppError {
                message: format!(
                    "Restore failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<SchoolBackup, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<SchoolBackup>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Backup not found".into(),
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
    ) -> Result<Paginated<SchoolBackup>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "backup_name",
            "_id",
            "school_id",
            "status",
            "backup_type",
            "created_by",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<SchoolBackup>(filter, &searchable, limit, skip, extra_match)
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
    ) -> Result<Paginated<SchoolBackupWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &[
                    "backup_name",
                    "_id",
                    "school_id",
                    "status",
                    "backup_type",
                    "created_by",
                ],
            );

            match_stage.extend(search);
        }

        let pipeline = backup_pipeline(match_stage);

        repo.aggregate_with_paginate::<SchoolBackupWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<SchoolBackupWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<SchoolBackupWithRelations>(backup_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Backup not found".into(),
            })
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<SchoolBackup, AppError> {
        let backup = self.find_one(Some(id), None).await?;

        // Delete backup file
        if std::path::Path::new(&backup.file_path).exists() {
            std::fs::remove_file(&backup.file_path).ok();
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.delete_one(id).await?;

        Ok(backup)
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count_backups(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "backup_name",
            "school_id",
            "status",
            "backup_type",
            "created_by",
        ];

        repo.count(filter, &searchable, extra_match).await
    }
}
