use chrono::{DateTime, Utc};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Database,
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{auth_user::AuthUserDto, common_details::Paginated},
    errors::AppError,
    services::audit_log_service::AuditLogService,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletedEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: Option<String>,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
    pub deleted_by_name: Option<String>,
    pub school_id: String,
}

pub struct RecycleBinService {
    pub db: Database,
}

impl RecycleBinService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get all deleted entities across collections
    pub async fn get_deleted_entities(
        &self,
        entity_type: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        school_id: ObjectId,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<DeletedEntity>, AppError> {
        let mut deleted_entities = Vec::new();

        // Collections to check for soft deletes
        let collections = vec![
            "students",
            "teachers",
            "school_staff",
            "parents",
            "classes",
            "announcements",
            "assignments",
            "exams",
        ];

        for collection_name in collections {
            // Skip if entity_type filter doesn't match
            if let Some(ref filter_type) = entity_type {
                if filter_type != collection_name {
                    continue;
                }
            }

            let collection = self.db.collection::<Document>(collection_name);

            let mut filter = doc! {
                "deleted_at": { "$ne": null },
                "school_id": school_id
            };

            // Add date range filters
            if let Some(start) = start_date {
                filter.insert("deleted_at", doc! { "$gte": mongodb::bson::to_bson(&start).unwrap() });
            }
            if let Some(end) = end_date {
                let mut deleted_at_filter = filter
                    .get_document("deleted_at")
                    .cloned()
                    .unwrap_or_default();
                deleted_at_filter.insert("$lte", mongodb::bson::to_bson(&end).unwrap());
                filter.insert("deleted_at", deleted_at_filter);
            }

            let mut cursor = collection
                .find(filter)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to query {}: {}", collection_name, e),
                })?;

            use futures::stream::StreamExt;

            while let Some(result) = cursor.next().await {
                match result {
                    Ok(doc) => {
                        let entity_id = doc
                            .get_object_id("_id")
                            .map(|id| id.to_hex())
                            .unwrap_or_default();

                        let entity_name = doc
                            .get_str("name")
                            .ok()
                            .or_else(|| doc.get_str("title").ok())
                            .map(|s| s.to_string());

                        let deleted_at = doc
                            .get_datetime("deleted_at")
                            .ok()
                            .map(|dt| {
                                use chrono::TimeZone;
                                Utc.timestamp_opt(dt.timestamp_millis() / 1000, 0).unwrap()
                            })
                            .unwrap_or_else(Utc::now);

                        let deleted_by = doc
                            .get_object_id("deleted_by")
                            .map(|id| id.to_hex())
                            .unwrap_or_default();

                        deleted_entities.push(DeletedEntity {
                            entity_type: collection_name.to_string(),
                            entity_id,
                            entity_name,
                            deleted_at,
                            deleted_by,
                            deleted_by_name: None,
                            school_id: school_id.to_hex(),
                        });
                    }
                    Err(e) => {
                        eprintln!("Error reading document from {}: {}", collection_name, e);
                    }
                }
            }
        }

        // Sort by deleted_at descending
        deleted_entities.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));

        // Apply pagination
        let total = deleted_entities.len() as i64;
        let skip_val = skip.unwrap_or(0) as usize;
        let limit_val = limit.unwrap_or(20) as usize;

        let paginated_data: Vec<DeletedEntity> = deleted_entities
            .into_iter()
            .skip(skip_val)
            .take(limit_val)
            .collect();

        let total_pages = (total as f64 / limit_val as f64).ceil() as i64;
        let current_page = (skip_val / limit_val) as i64 + 1;

        Ok(Paginated {
            data: paginated_data,
            total,
            total_pages,
            current_page,
        })
    }

    /// Restore a soft-deleted entity
    pub async fn restore_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
        user: &AuthUserDto,
        state: &crate::config::state::AppState,
    ) -> Result<(), AppError> {
        let oid = ObjectId::parse_str(entity_id).map_err(|_| AppError {
            message: "Invalid entity ID".to_string(),
        })?;

        let collection = self.db.collection::<Document>(entity_type);

        // Check if entity exists and is deleted
        let entity = collection
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find entity: {}", e),
            })?
            .ok_or(AppError {
                message: "Entity not found".to_string(),
            })?;

        if entity.get("deleted_at").is_none() {
            return Err(AppError {
                message: "Entity is not deleted".to_string(),
            });
        }

        // Restore by removing deleted_at and deleted_by fields
        collection
            .update_one(
                doc! { "_id": oid },
                doc! {
                    "$unset": {
                        "deleted_at": "",
                        "deleted_by": ""
                    }
                },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to restore entity: {}", e),
            })?;

        // Log audit event
        let school_id = entity
            .get_object_id("school_id")
            .ok()
            .unwrap_or_else(|| ObjectId::new());

        let audit_service = AuditLogService::new(&state.db.main_db());
        audit_service
            .log_event(
                school_id,
                user,
                &format!("{}.restore", entity_type),
                entity_type,
                oid,
                Some(doc! {
                    "restored_at": mongodb::bson::to_bson(&Utc::now()).unwrap(),
                }),
                None,
                None,
            )
            .await
            .ok();

        Ok(())
    }

    /// Permanently delete a soft-deleted entity
    pub async fn permanently_delete(
        &self,
        entity_type: &str,
        entity_id: &str,
        user: &AuthUserDto,
        state: &crate::config::state::AppState,
    ) -> Result<(), AppError> {
        let oid = ObjectId::parse_str(entity_id).map_err(|_| AppError {
            message: "Invalid entity ID".to_string(),
        })?;

        let collection = self.db.collection::<Document>(entity_type);

        // Check if entity exists and is deleted
        let entity = collection
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find entity: {}", e),
            })?
            .ok_or(AppError {
                message: "Entity not found".to_string(),
            })?;

        if entity.get("deleted_at").is_none() {
            return Err(AppError {
                message: "Entity is not deleted".to_string(),
            });
        }

        // Permanently delete
        collection
            .delete_one(doc! { "_id": oid })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to permanently delete entity: {}", e),
            })?;

        // Log audit event
        let school_id = entity
            .get_object_id("school_id")
            .ok()
            .unwrap_or_else(|| ObjectId::new());

        let audit_service = AuditLogService::new(&state.db.main_db());
        audit_service
            .log_event(
                school_id,
                user,
                &format!("{}.permanent_delete", entity_type),
                entity_type,
                oid,
                Some(doc! {
                    "permanently_deleted_at": mongodb::bson::to_bson(&Utc::now()).unwrap(),
                }),
                None,
                None,
            )
            .await
            .ok();

        Ok(())
    }
}
