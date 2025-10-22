use crate::{
    domain::trade::{Trade, TradeWithOthers, UpdateTrade},
    models::id_model::IdType,
    repositories::trade_repo::TradeRepo,
    services::sector_service::SectorService,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct TradeService<'a> {
    repo: &'a TradeRepo,
}

impl<'a> TradeService<'a> {
    pub fn new(repo: &'a TradeRepo) -> Self {
        Self { repo }
    }

    /// Get all trades
    pub async fn get_all_trades(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Trade>, String> {
        self.repo
            .get_all_trades(filter, limit, skip)
            .await
            .map_err(|e| e.message)
    }

    pub async fn get_all_trades_with_others(&self) -> Result<Vec<TradeWithOthers>, String> {
        self.repo
            .get_all_trades_with_others()
            .await
            .map_err(|e| e.message)
    }

    /// Create a new trade
    pub async fn create_trade(
        &self,
        mut new_trade: Trade,
        sector_service: &SectorService<'a>,
    ) -> Result<Trade, String> {
        // Check username uniqueness
        if let Ok(Some(_)) = self.repo.find_by_username(&new_trade.username).await {
            return Err("username already exists".to_string());
        }

        // ✅ Check sector existence
        if let Some(ref sector_id) = new_trade.sector_id {
            let sector_id_type = IdType::from_object_id(*sector_id);
            sector_service
                .get_sector_by_id(&sector_id_type)
                .await
                .map_err(|_| "Sector not found".to_string())?;
        }

        // ✅ Parent trade validation
        if let Some(ref parent_trade_id) = new_trade.trade_id {
            let parent_id_type = IdType::from_object_id(*parent_trade_id);
            self.repo
                .find_by_id(&parent_id_type)
                .await
                .map_err(|e| e.message.to_string())?
                .ok_or("Parent trade not found".to_string())?;
        }

        // Set timestamps
        let now = Some(Utc::now());
        new_trade.created_at = now;
        new_trade.updated_at = now;

        // Ensure Mongo generates id
        new_trade.id = Some(ObjectId::new());

        // Save trade in database
        let inserted_trade = self
            .repo
            .insert_trade(&new_trade)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted_trade)
    }

    /// Get trade by ID
    pub async fn get_trade_by_id(&self, id: &IdType) -> Result<Trade, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Trade not found".to_string())
    }

    /// Get trade by username
    pub async fn get_trade_by_username(&self, username: &str) -> Result<Trade, String> {
        self.repo
            .find_by_username(username)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Trade not found".to_string())
    }

    pub async fn get_trade_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<TradeWithOthers, String> {
        self.repo
            .find_by_id_with_others(id)
            .await
            .map_err(|e| e.message)?
            .ok_or_else(|| "Trade not found".to_string())
    }

    pub async fn get_trade_by_username_with_sector(
        &self,
        username: &str,
    ) -> Result<TradeWithOthers, String> {
        self.repo
            .find_by_username_with_others(username)
            .await
            .map_err(|e| e.message)?
            .ok_or_else(|| "Trade not found".to_string())
    }

    /// Update a trade by id
    pub async fn update_trade(
        &self,
        id: &IdType,
        mut updated_data: UpdateTrade,
        sector_service: &SectorService<'a>,
    ) -> Result<Trade, String> {
        let trade_to_update = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Trade not found".to_string())?;

        // Check username uniqueness if updated
        if let Some(ref username) = updated_data.username {
            if trade_to_update.username != *username {
                if let Ok(Some(_)) = self.repo.find_by_username(username).await {
                    return Err("username already exists".to_string());
                }
            }
        }

        // ✅ Check sector existence if sector_id is updated
        if let Some(ref sector_id) = updated_data.sector_id {
            let sector_id_type = IdType::from_object_id(*sector_id);
            sector_service
                .get_sector_by_id(&sector_id_type)
                .await
                .map_err(|_| "Sector not found".to_string())?;
        }

        // ✅ Parent trade validation
        if let Some(ref parent_trade_id) = updated_data.trade_id {
            let parent_id_type = IdType::from_object_id(*parent_trade_id);
            self.repo
                .find_by_id(&parent_id_type)
                .await
                .map_err(|e| e.message.to_string())?
                .ok_or("Parent trade not found".to_string())?;

            // Prevent setting itself as its own parent
            if trade_to_update.id == Some(*parent_trade_id) {
                return Err("A trade cannot be its own parent".to_string());
            }
        }

        // Update timestamp
        updated_data.updated_at = Some(Utc::now());

        let updated_trade = self
            .repo
            .update_trade(id, &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_trade)
    }

    /// Delete a trade by id
    pub async fn delete_trade(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_trade(id).await.map_err(|e| e.message)
    }

    /// Get multiple trades by their IDs
    pub async fn get_trades_by_ids(&self, trade_ids: &[ObjectId]) -> Result<Vec<Trade>, String> {
        self.repo
            .get_trades_by_ids(trade_ids)
            .await
            .map_err(|e| e.message)
    }

    /// Get trades by sector IDs
    pub async fn get_trades_by_sector_ids(
        &self,
        sector_ids: &[ObjectId],
    ) -> Result<Vec<Trade>, String> {
        self.repo
            .get_trades_by_sector_ids(sector_ids)
            .await
            .map_err(|e| e.message)
    }

    // NEW METHODS ADDED HERE:

    /// Get trades by a specific sector ID
    pub async fn get_trades_by_sector_id(&self, id: &IdType) -> Result<Vec<Trade>, String> {
        self.repo
            .get_trades_by_sector_id(id)
            .await
            .map_err(|e| e.message)
    }

    /// Get trades by a specific parent trade ID (self-relation)
    pub async fn get_trades_by_trade_id(&self, id: &IdType) -> Result<Vec<Trade>, String> {
        self.repo
            .get_trades_by_trade_id(id)
            .await
            .map_err(|e| e.message)
    }
}
