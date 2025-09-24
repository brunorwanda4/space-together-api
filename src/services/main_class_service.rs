use crate::{
    domain::main_class::{MainClass, MainClassWithOthers, UpdateMainClass},
    models::id_model::IdType,
    repositories::main_class_repo::MainClassRepo,
    services::trade_service::TradeService,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct MainClassService<'a> {
    repo: &'a MainClassRepo,
    trade_service: &'a TradeService<'a>,
}

impl<'a> MainClassService<'a> {
    pub fn new(repo: &'a MainClassRepo, trade_service: &'a TradeService<'a>) -> Self {
        Self {
            repo,
            trade_service,
        }
    }

    /// Get all main classes
    pub async fn get_all(&self) -> Result<Vec<MainClass>, String> {
        self.repo.get_all().await.map_err(|e| e.message)
    }

    /// Create a new main class
    pub async fn create_main_class(&self, mut new_class: MainClass) -> Result<MainClass, String> {
        // Check username uniqueness
        if let Ok(Some(_)) = self.repo.find_by_username(&new_class.username).await {
            return Err("username already exists".to_string());
        }

        // ✅ Trade existence check
        if let Some(trade_id) = new_class.trade_id {
            let trade_id_type = IdType::from_object_id(trade_id);
            self.trade_service
                .get_trade_by_id(&trade_id_type)
                .await
                .map_err(|_| "Associated trade not found".to_string())?;
        }

        // Set timestamps
        let now = Some(Utc::now());
        new_class.created_at = now;
        new_class.updated_at = now;

        // Ensure Mongo generates id
        new_class.id = Some(ObjectId::new());

        // Insert in DB
        let inserted = self
            .repo
            .insert_main_class(&new_class)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted)
    }

    /// Get main class by ID
    pub async fn get_by_id(&self, id: &IdType) -> Result<MainClass, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "MainClass not found".to_string())
    }

    /// Get main class by username
    pub async fn get_by_username(&self, username: &str) -> Result<MainClass, String> {
        self.repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "MainClass not found".to_string())
    }

    pub async fn get_by_id_with_others(&self, id: &IdType) -> Result<MainClassWithOthers, String> {
        self.repo
            .find_by_id_with_others(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "MainClass not found".to_string())
    }

    pub async fn get_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<MainClassWithOthers, String> {
        self.repo
            .find_by_username_with_others(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "MainClass not found".to_string())
    }

    /// Update main class
    pub async fn update(
        &self,
        id: &IdType,
        mut updated_data: UpdateMainClass,
    ) -> Result<MainClass, String> {
        let current = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "MainClass not found".to_string())?;

        // Check username uniqueness if updated
        if let Some(ref username) = updated_data.username {
            if current.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("username already exists".to_string());
                }
            }
        }

        // ✅ Trade existence if updated
        if let Some(trade_id) = updated_data.trade_id {
            let trade_id_type = IdType::from_object_id(trade_id);
            self.trade_service
                .get_trade_by_id(&trade_id_type)
                .await
                .map_err(|_| "Associated trade not found".to_string())?;
        }

        // Update timestamp
        updated_data.updated_at = Some(Utc::now());

        let updated = self
            .repo
            .update_main_class(id, &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(updated)
    }

    /// Delete main class
    pub async fn delete(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_main_class(id).await.map_err(|e| e.message)
    }
}
