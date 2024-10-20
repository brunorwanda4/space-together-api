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
) -> TradingResult<TradingModelGet> {
    let find_by_username = query
        .db
        .trading_db
        .get_trading_by_username(trading.username.clone())
        .await;
    let find_by_code = query
        .db
        .trading_db
        .get_trading_by_code(trading.code.clone())
        .await;

    if find_by_code.is_ok() {
        return Err(TradingErr::TradingCodeIsReadyExit);
    }

    if find_by_username.is_ok() {
        return Err(TradingErr::TradingUsernameIsReadyExit);
    };

    let new = query.db.trading_db.create_trading(trading).await;

    match new {
        Ok(result) => {
            let id = if let Bson::ObjectId(oid) = result.inserted_id {
                Ok(oid.to_hex()) // Convert ObjectId to hex string
            } else {
                Err(TradingErr::CanNotCreateTrading)
            };
            let get = query.db.trading_db.get_trading_by_id(id.unwrap()).await;
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
                Err(e) => Err(e),
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
        Ok(res) => {
            let get = query.db.trading_db.get_trading_by_id(res).await;
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

pub async fn get_trading_controller(
    id: String,
    query: Arc<AppState>,
) -> TradingResult<TradingModelGet> {
    let get = query.db.trading_db.get_trading_by_id(id.clone()).await;
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

pub async fn get_all_tradings_controllers(
    query: Arc<AppState>,
) -> TradingResult<Vec<TradingModelGet>> {
    let get_all = query.db.trading_db.get_all_tradings().await;

    match get_all {
        Ok(result) => {
            let tradings_get: Vec<TradingModelGet> = result
                .into_iter()
                .map(|trading| TradingModelGet {
                    id: trading
                        .id
                        .map_or("".to_string(), |obj_id| obj_id.to_string()), // Convert ObjectId to String
                    name: trading.name,
                    username: trading.username,
                    code: trading.code,
                    trading_type: trading.trading_type,
                    schools_id: trading
                        .schools_id
                        .map(|ids| ids.iter().map(|id| id.to_string()).collect()), // Vec<ObjectId> to Vec<String>
                    description: trading.description,
                    is_active: trading.is_active,
                    education: trading.education,
                    reasons: trading
                        .reasons
                        .map(|ids| ids.iter().map(|id| id.to_string()).collect()), // Vec<ObjectId> to Vec<String>
                    created_at: trading.created_at.to_rfc3339_string(), // Convert DateTime to String
                    updated_at: trading.updated_at.map(|dt| dt.to_rfc3339_string()), // Convert Option<DateTime> to Option<String>
                })
                .collect();

            Ok(tradings_get)
        }
        Err(err) => Err(err),
    }
}
