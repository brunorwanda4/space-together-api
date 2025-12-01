use chrono::Utc;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

use crate::{
    domain::template_subject::TemplateSubject, errors::AppError, models::id_model::IdType,
    repositories::base_repo::BaseRepository,
};

pub struct TemplateSubjectRepo {
    pub collection: Collection<TemplateSubject>,
}

impl TemplateSubjectRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<TemplateSubject>("template_subjects"),
        }
    }

    // =========================================================
    // 🔹 Utility Helpers
    // =========================================================

    fn obj_id(id: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(id).map_err(|e| AppError {
            message: format!("Invalid ObjectId: {}", e),
        })
    }

    async fn find_one(&self, filter: Document) -> Result<Option<TemplateSubject>, AppError> {
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Database query failed: {}", e),
            })
    }

    async fn update_and_fetch(
        &self,
        filter: Document,
        update: Document,
    ) -> Result<TemplateSubject, AppError> {
        self.collection
            .update_one(filter.clone(), update)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update document: {}", e),
            })?;

        self.find_one(filter).await?.ok_or(AppError {
            message: "Document not found after update".into(),
        })
    }

    // =========================================================
    // 🔹 Create
    // =========================================================

    pub async fn create(&self, dto: &mut TemplateSubject) -> Result<TemplateSubject, AppError> {
        // Add unique index
        let index = IndexModel::builder()
            .keys(doc! { "code": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create unique index for code: {}", e),
            })?;

        dto.id = None;
        dto.created_at = Some(Utc::now());
        dto.updated_at = Some(Utc::now());

        self.collection
            .insert_one(dto.clone())
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert subject: {}", e),
            })?;

        let inserted = self
            .collection
            .find_one(doc! { "code": &dto.code })
            .await
            .map_err(|e| AppError {
                message: format!("Error fetching new subject: {}", e),
            })?;

        inserted.ok_or(AppError {
            message: "Inserted subject not found".into(),
        })
    }

    // =========================================================
    // 🔹 Get by ID
    // =========================================================

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<TemplateSubject>, AppError> {
        let obj = IdType::to_object_id(id)?;
        self.find_one(doc! { "_id": obj }).await
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<(Vec<TemplateSubject>, i64, i64, i64), AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "code", "description"];

        base_repo
            .get_all::<TemplateSubject>(filter, &searchable, limit, skip, extra_match)
            .await
    }

    pub async fn update_fields(
        &self,
        id: &str,
        update: &TemplateSubject,
    ) -> Result<TemplateSubject, AppError> {
        let obj_id = Self::obj_id(id)?;

        let full = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        let mut data = Document::new();
        for (k, v) in full {
            if !matches!(v, bson::Bson::Null) {
                data.insert(k, v);
            }
        }

        if data.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        data.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        self.update_and_fetch(doc! { "_id": obj_id }, doc! { "$set": data })
            .await
    }

    // =========================================================
    // 🔹 Delete
    // =========================================================

    pub async fn delete(&self, id: &IdType) -> Result<(), AppError> {
        let obj = Self::obj_id(&id.as_string())?;

        let res = self
            .collection
            .delete_one(doc! { "_id": obj })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete subject: {}", e),
            })?;

        if res.deleted_count == 0 {
            Err(AppError {
                message: "Subject not found".into(),
            })
        } else {
            Ok(())
        }
    }
}
