use std::sync::Arc;

use crate::{
    error::school::trading_error::{TradingErr, TradingResult},
    models::school::trading_model::{TradingModelGet, TradingModelNew, TradingModelUpdate},
    AppState,
};
use mongodb::bson::Bson;

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

pub async fn update_trading_controller(
    id: String,
    trading: TradingModelUpdate,
    query: Arc<AppState>,
) -> TradingResult<TradingModelGet> {
    let update = query
        .db
        .trading_db
        .update_trading(id.clone(), trading)
        .await;

    match update {
        Ok(_) => {
            let get = query.db.trading_db.get_trading_by_id(id).await;

            match get {
                Ok(result) => {
                    let get_trading_model = TradingModelGet {
                        id: result
                            .id
                            .map_or("".to_string(), |obj_id| obj_id.to_string()), // Convert ObjectId to String
                        name: result.name,
                        username: result.username,
                        code: result.code,
                        trading_type: result.trading_type,
                        schools_id: result
                            .schools_id
                            .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
                        description: result.description,
                        is_active: result.is_active,
                        education: result.education,
                        reasons: result
                            .reasons
                            .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
                        created_at: result.created_at.to_rfc3339_string(),
                        updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
                    };
                    Ok(get_trading_model)
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
