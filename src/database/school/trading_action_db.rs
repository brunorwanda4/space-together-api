use mongodb::{results::InsertOneResult, Collection};

use crate::{
    error::school::trading_error::{TradingErr, TradingResult},
    models::school::trading_model::{TradingModel, TradingModelNew},
};

#[derive(Debug, Clone)]
pub struct TradingActionDb {
    pub trading: Collection<TradingModel>,
}

impl TradingActionDb {
    // Implement methods for CRUD operations on TradingModel
    pub async fn create_trading(&self, trading: TradingModelNew) -> TradingResult<InsertOneResult> {
        let new = TradingModel::new(trading);
        let create = self.trading.insert_one(new).await;

        match create {
            Ok(result) => Ok(result),
            Err(_) => Err(TradingErr::CanNotCreateTrading),
        }
    }
}
