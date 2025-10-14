use crate::{
    domain::main_class::{MainClass, MainClassWithOthers, MainClassWithTrade, UpdateMainClass},
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

    pub async fn get_all_with_trade(&self) -> Result<Vec<MainClassWithTrade>, String> {
        self.repo.get_all_with_trade().await.map_err(|e| e.message)
    }

    pub async fn get_with_trade_id(
        &self,
        trade_id: &IdType,
    ) -> Result<Vec<MainClassWithTrade>, String> {
        self.repo
            .find_by_trade_id(trade_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get all main classes
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        trade_id: Option<String>,
    ) -> Result<Vec<MainClass>, String> {
        self.repo
            .get_all(filter, limit, skip, trade_id)
            .await
            .map_err(|e| e.message)
    }

    /// Create a new main class
    pub async fn create_main_class(&self, mut new_class: MainClass) -> Result<MainClass, String> {
        // 1️⃣ Check username uniqueness
        if let Ok(Some(_)) = self.repo.find_by_username(&new_class.username).await {
            return Err("Username already exists".to_string());
        }

        // 2️⃣ Validate trade existence and level range
        if let Some(trade_id) = new_class.trade_id {
            let trade_id_type = IdType::from_object_id(trade_id);
            let trade = self
                .trade_service
                .get_trade_by_id(&trade_id_type)
                .await
                .map_err(|_| "Associated trade not found".to_string())?;

            // Ensure level is provided
            let level = new_class
                .level
                .ok_or_else(|| "Level is required when trade_id is provided".to_string())?;

            // Validate level range
            if level < trade.class_min || level > trade.class_max {
                return Err(format!(
                    "Level {} must be between {} and {} for this trade.",
                    level, trade.class_min, trade.class_max
                ));
            }

            // 3️⃣ Check for existing main class with same trade + level
            if let Ok(Some(_existing)) = self.repo.find_by_trade_and_level(&trade_id, level).await {
                return Err(format!(
                    "A main class with trade_id '{}' and level {} already exists.",
                    trade.username, level
                ));
            }
        }

        // 4️⃣ Set timestamps
        let now = Some(Utc::now());
        new_class.created_at = now;
        new_class.updated_at = now;
        new_class.id = Some(ObjectId::new());

        // 5️⃣ Insert into DB
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
        updated_data: UpdateMainClass,
    ) -> Result<MainClass, String> {
        // 1️⃣ Fetch existing record
        let mut existing = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "MainClass not found".to_string())?;

        // 2️⃣ Handle username uniqueness
        if let Some(ref new_username) = updated_data.username {
            if existing.username != *new_username {
                if let Ok(Some(_)) = self.repo.find_by_username(new_username).await {
                    return Err("Username already exists".to_string());
                }
            }
        }

        // 3️⃣ Determine target trade & level
        let trade_id_to_check = updated_data
            .trade_id
            .unwrap_or_else(|| existing.trade_id.expect("MainClass must have trade_id"));

        let level_to_check = updated_data
            .level
            .unwrap_or_else(|| existing.level.expect("MainClass must have level"));

        let trade_id_type = IdType::from_object_id(trade_id_to_check);
        let trade = self
            .trade_service
            .get_trade_by_id(&trade_id_type)
            .await
            .map_err(|_| "Associated trade not found".to_string())?;

        // 4️⃣ Validate level range
        if level_to_check < trade.class_min || level_to_check > trade.class_max {
            return Err(format!(
                "Level {} must be between {} and {} for this trade.",
                level_to_check, trade.class_min, trade.class_max
            ));
        }

        // 5️⃣ Check duplicate trade+level (skip if same record)
        if let Ok(Some(existing_class)) = self
            .repo
            .find_by_trade_and_level(&trade_id_to_check, level_to_check)
            .await
        {
            if existing_class.id != existing.id {
                return Err(format!(
                    "A main class with trade '{}' and level {} already exists.",
                    trade.username, level_to_check
                ));
            }
        }

        // 6️⃣ Apply updates
        if let Some(name) = updated_data.name {
            existing.name = name;
        }
        if let Some(username) = updated_data.username {
            existing.username = username;
        }
        if let Some(description) = updated_data.description {
            existing.description = Some(description);
        }
        if let Some(disable) = updated_data.disable {
            existing.disable = Some(disable);
        }
        existing.trade_id = Some(trade_id_to_check);
        existing.level = Some(level_to_check);

        existing.updated_at = Some(Utc::now());

        let updated = self
            .repo
            .update_main_class(id, &existing)
            .await
            .map_err(|e| e.message)?;

        Ok(updated)
    }

    /// Delete main class
    pub async fn delete(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_main_class(id).await.map_err(|e| e.message)
    }
}
