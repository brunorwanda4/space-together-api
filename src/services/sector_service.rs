use crate::{
    domain::sector::{Sector, UpdateSector},
    models::id_model::IdType,
    repositories::sector_repo::SectorRepo,
    services::cloudinary_service::CloudinaryService,
    utils::names::is_valid_username,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SectorService<'a> {
    repo: &'a SectorRepo,
}

impl<'a> SectorService<'a> {
    pub fn new(repo: &'a SectorRepo) -> Self {
        Self { repo }
    }

    /// Get all sectors
    pub async fn get_all_sectors(&self) -> Result<Vec<Sector>, String> {
        self.repo.get_all_sectors().await.map_err(|e| e.message)
    }

    /// Create a new sector
    pub async fn create_sector(&self, mut new_sector: Sector) -> Result<Sector, String> {
        is_valid_username(&new_sector.username)?;

        if let Ok(Some(_)) = self.repo.find_by_username(&new_sector.username).await {
            return Err("username already exists".to_string());
        }

        if let Ok(Some(_)) = self.repo.find_by_username(&new_sector.username).await {
            return Err("username already exists".to_string());
        }
        // ✅ Handle logo (upload to Cloudinary if provided)
        if let Some(new_logo_file) = new_sector.logo.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&new_logo_file).await?;
            new_sector.logo_id = Some(cloud_res.public_id);
            new_sector.logo = Some(cloud_res.secure_url);
        }

        // Set timestamps
        let now = Some(Utc::now());
        new_sector.created_at = now;
        new_sector.updated_at = now;

        // Ensure Mongo generates id
        new_sector.id = Some(ObjectId::new());

        // Save sector in database
        let inserted_sector = self
            .repo
            .insert_sector(&new_sector)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted_sector)
    }

    /// Get sector by ID
    pub async fn get_sector_by_id(&self, id: &IdType) -> Result<Sector, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Sector not found".to_string())
    }

    pub async fn get_sector_by_username(&self, username: &str) -> Result<Sector, String> {
        self.repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Sector not found".to_string())
    }

    /// Update a sector by id
    pub async fn update_sector(
        &self,
        id: &IdType,
        updated_data: UpdateSector,
    ) -> Result<Sector, String> {
        if let Some(ref username) = updated_data.username {
            is_valid_username(username)?;
        }

        let mut sector_to_update = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Sector not found".to_string())?;

        if let Some(ref username) = updated_data.username {
            if sector_to_update.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("username already exists".to_string());
                }
            }
        }

        // ✅ Handle logo update
        if let Some(ref new_logo) = updated_data.logo {
            if let Some(old_logo) = sector_to_update.logo_id.clone() {
                CloudinaryService::delete_from_cloudinary(&old_logo)
                    .await
                    .ok();
            }
            let cloud_res = CloudinaryService::upload_to_cloudinary(new_logo).await?;
            sector_to_update.logo_id = Some(cloud_res.public_id);
            sector_to_update.logo = Some(cloud_res.secure_url);
        }

        // ✅ Only overwrite if provided
        if let Some(ref name) = updated_data.name {
            sector_to_update.name = name.clone();
        }
        if let Some(ref username) = updated_data.username {
            sector_to_update.username = username.clone();
        }
        if let Some(ref description) = updated_data.description {
            sector_to_update.description = Some(description.clone());
        }
        if let Some(curriculum) = updated_data.curriculum {
            sector_to_update.curriculum = Some(curriculum);
        }
        if let Some(ref country) = updated_data.country {
            sector_to_update.country = country.clone();
        }
        if let Some(ref disable) = updated_data.disable {
            sector_to_update.disable = Some(*disable);
        }

        if let Some(ref t) = updated_data.r#type {
            sector_to_update.r#type = t.clone();
        }

        sector_to_update.updated_at = Some(Utc::now());

        let updated_sector = self
            .repo
            .update_sector(id, &updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_sector)
    }

    /// Delete a sector by id
    pub async fn delete_sector(&self, id: &IdType) -> Result<(), String> {
        let sector = self.repo.find_by_id(id).await.map_err(|e| e.message)?;

        if let Some(delete_sector) = sector {
            if let Some(logo_public_id) = delete_sector.logo_id {
                CloudinaryService::delete_from_cloudinary(&logo_public_id)
                    .await
                    .ok(); // ignore delete errors
            }
        }

        self.repo.delete_sector(id).await.map_err(|e| e.message)
    }
}
