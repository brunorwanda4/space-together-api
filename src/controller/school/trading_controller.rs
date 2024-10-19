use std::sync::Arc;

use mongodb::bson::Bson;

use crate::{
    error::school::trading_error::{TradingErr, TradingResult},
    models::school::trading_model::TradingModelNew,
    AppState,
};

pub async fn create_trading_controller(
    trading: TradingModelNew,
    query: Arc<AppState>,
) -> TradingResult<String> {
    let new = query.db.trading_db.create_trading(trading).await;

    match new {
        Ok(result) => {
            if let Bson::ObjectId(oid) = result.inserted_id {
                Ok(oid.to_hex()) // Convert ObjectId to hex string
            } else {
                Err(TradingErr::CanNotCreateTrading)
            }
        }
        Err(err) => Err(err),
    }
}
