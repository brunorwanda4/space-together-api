use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        sector::{Sector, SectorPartial},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::base_repo::BaseRepository,
    services::cloudinary_service::CloudinaryService,
    utils::mongo_utils::extract_valid_fields,
};

pub struct SectorService {
    pub collection: Collection<Sector>,
}

impl SectorService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Sector>("sectors"),
        }
    }

    // =========================
    // INDEXES
    // =========================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", true),
            IndexDef::single("username", true),
            IndexDef::single("country", false),
            IndexDef::single("type", false),
            IndexDef::single("disable", false),
            IndexDef::compound(vec![("country", 1), ("type", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: Sector) -> Result<Sector, AppError> {
        self.ensure_indexes().await?;

        // unique name
        if let Ok(existing) = self.find_one(None, Some(doc! { "name": &dto.name })).await {
            return Err(AppError {
                message: format!("Sector name already exists: {}", existing.name),
            });
        }

        // unique username
        if let Ok(existing) = self
            .find_one(None, Some(doc! { "username": &dto.username }))
            .await
        {
            return Err(AppError {
                message: format!("Sector username already exists: {}", existing.username),
            });
        }

        let mut new_sector = dto.clone();

        if let Some(new_logo_file) = new_sector.logo.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&new_logo_file)
                .await
                .map_err(|e| AppError { message: e })?;

            new_sector.logo_id = Some(cloud_res.public_id);
            new_sector.logo = Some(cloud_res.secure_url);
        }

        let full_doc = bson::to_document(&new_sector).map_err(|e| AppError {
            message: format!("Failed to serialize sector: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.create::<Sector>(extract_valid_fields(full_doc), None)
            .await
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Sector, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Sector>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Sector not found".into(),
            })
    }

    // =========================
    // GET ALL (PAGINATED)
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Sector>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "country", "type", "description", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Sector>(filter, &searchable, limit, skip, extra_match)
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
    pub async fn update(&self, id: &IdType, update: &SectorPartial) -> Result<Sector, AppError> {
        let existing = self.find_one(Some(id), None).await?;

        // name uniqueness
        if let Some(ref name) = update.name {
            if existing.name != *name {
                if let Ok(_) = self.find_one(None, Some(doc! { "name": name })).await {
                    return Err(AppError {
                        message: format!("Sector name already exists: {}", name),
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
                        message: format!("Sector username already exists: {}", username),
                    });
                }
            }
        }

        let existing_sector = self.find_one(Some(id), None).await?;

        let mut update_data = update.clone();

        if let Some(new_logo) = update_data.logo.clone().flatten() {
            if Some(new_logo.clone()) != existing_sector.logo {
                if let Some(old_logo_id) = existing_sector.logo_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_logo_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_logo)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.logo_id = Some(Some(cloud_res.public_id));
                update_data.logo = Some(Some(cloud_res.secure_url));
            }
        }

        let full_doc = bson::to_document(&update_data).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<Sector>(id, extract_valid_fields(full_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Sector, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let sector = self.find_one(Some(id), None).await?;

        if let Some(ref logo_id) = sector.logo_id {
            CloudinaryService::delete_from_cloudinary(logo_id)
                .await
                .ok();
        }

        repo.delete_one(id).await?;

        Ok(sector)
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

        let searchable = ["name", "username", "country", "type", "description", "_id"];

        repo.count(filter, &searchable, extra_match).await
    }

    pub async fn find_by_ids(&self, ids: Vec<IdType>) -> Result<Vec<Sector>, AppError> {
        // Convert string IDs into MongoDB ObjectIds
        let object_ids: Vec<ObjectId> = ids
            .into_iter()
            .filter_map(|id| ObjectId::parse_str(id.as_string()).ok())
            .collect();

        if object_ids.is_empty() {
            return Ok(vec![]); // No valid IDs — return empty list
        }

        // Build the query to match multiple IDs
        let filter = doc! { "_id": { "$in": object_ids } };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to fetch sectors by ids: {}", e),
        })?;

        let mut sectors = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate sectors: {}", e),
        })? {
            sectors.push(result);
        }

        Ok(sectors)
    }
}
