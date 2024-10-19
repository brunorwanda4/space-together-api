use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    controller::school::trading_controller::{
        create_trading_controller, update_trading_controller,
    },
    error::res_req::ResReq,
    models::school::trading_model::{TradingModelNew, TradingModelUpdate},
    AppState,
};

pub async fn create_trading_handler(
    State(query): State<Arc<AppState>>,
    Json(trading): Json<TradingModelNew>,
) -> impl IntoResponse {
    let req = create_trading_controller(trading, query).await;

    match req {
        Ok(res) => {
            let data = ResReq {
                success: true,
                message: res.to_string(),
            };
            (StatusCode::OK, Json(data)).into_response()
        }
        Err(err) => {
            let data = ResReq {
                success: false,
                message: err.to_string(),
            };
            (StatusCode::CREATED, Json(data)).into_response()
        }
    }
}

pub async fn update_trading_handle(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(trading): Json<TradingModelUpdate>,
) -> impl IntoResponse {
    let req = update_trading_controller(id, trading, query).await;

    match req {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}
