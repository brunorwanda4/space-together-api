use actix_web::App;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        school::{School, SchoolPartial},
    },
    errors::AppError,
    mappers::school_mapper::to_school_school_token,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::base_repo::BaseRepository,
    utils::{
        mongo_utils::extract_valid_fields,
        school_token::{create_school_token, verify_school_token},
    },
};

pub struct SchoolService {
    pub collection: Collection<School>,
}

impl SchoolService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<School>("schools"),
        }
    }

    // =========================
    // INDEXES
    // =========================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", true),
            IndexDef::single("username", true),
            IndexDef::single("code", true),
            IndexDef::single("creator_id", false),
            IndexDef::single("is_active", false),
            IndexDef::compound(vec![("username", 1), ("is_active", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: School) -> Result<School, AppError> {
        self.ensure_indexes().await?;

        // unique name
        if let Ok(existing) = self.find_one(None, Some(doc! { "name": &dto.name })).await {
            return Err(AppError {
                message: format!("School name already exists: {}", existing.name),
            });
        }

        // unique username
        if let Ok(existing) = self
            .find_one(None, Some(doc! { "username": &dto.username }))
            .await
        {
            return Err(AppError {
                message: format!("School username already exists: {}", existing.username),
            });
        }

        // unique code (optional)
        if let Some(ref code) = dto.code {
            if let Ok(_) = self.find_one(None, Some(doc! { "code": code })).await {
                return Err(AppError {
                    message: format!("School code already exists: {}", code),
                });
            }
        }

        let mut new_school = dto.clone();
        new_school.created_at = Some(chrono::Utc::now());
        new_school.is_active = Some(true);

        let full_doc = bson::to_document(&new_school).map_err(|e| AppError {
            message: format!("Failed to serialize School: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<School>(extract_valid_fields(full_doc), None)
            .await
    }

    // =========================
    // FIND ONE (NO RELATIONS)
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<School, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<School>(filter, None)
            .await?
            .ok_or(AppError {
                message: "School not found".into(),
            })
    }

    // =========================
    // GET ALL
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<School>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "description", "code", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<School>(filter, &searchable, limit, skip, extra_match)
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
    pub async fn update(&self, id: &IdType, update: &SchoolPartial) -> Result<School, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        // name uniqueness
        if let Some(ref name) = update.name {
            if existing.name != *name {
                if let Ok(_) = self.find_one(None, Some(doc! { "name": name })).await {
                    return Err(AppError {
                        message: format!("School name already exists: {}", name),
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
                        message: format!("School username already exists: {}", username),
                    });
                }
            }
        }

        // code uniqueness
        if let Some(ref code) = update.code {
            if existing.code.as_deref() != Some(code) {
                if let Ok(_) = self.find_one(None, Some(doc! { "code": code })).await {
                    return Err(AppError {
                        message: format!("School code already exists: {}", code),
                    });
                }
            }
        }

        let mut update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<School>(id, extract_valid_fields(update_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<School, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let school = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(school)
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

        let searchable = ["name", "username", "description", "code", "_id"];

        repo.count(filter, &searchable, extra_match).await
    }

    pub async fn refresh_school_token(&self, token: &str) -> Result<String, AppError> {
        // remove "Bearer " if present
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_school_token(&token_clean).ok_or_else(|| AppError {
            message: "Invalid token".to_string(),
        })?;

        // get user from DB to ensure still valid
        let school_id = IdType::from_string(&claims.id);
        let school = self.find_one(Some(&school_id), None).await?;

        // create a fresh token
        let dto = to_school_school_token(&school).map_err(|e| AppError { message: e })?;
        let new_token = create_school_token(dto);

        Ok(new_token)
    }

    pub async fn create_school_token(&self, id: &IdType) -> Result<String, AppError> {
        let school = self.find_one(Some(id), None).await?;
        let school_token = to_school_school_token(&school).map_err(|e| AppError { message: e })?;
        let token = create_school_token(school_token);

        Ok(token)
    }
}
