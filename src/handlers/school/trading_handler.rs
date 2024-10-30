use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    controller::school::trading_controller::{
        create_trading_controller, get_all_tradings_controllers, get_trading_controller,
        update_trading_controller,
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
        Ok(res) => (StatusCode::CREATED, Json(res)).into_response(),
        Err(err) => {
            let data = ResReq {
                success: false,
                message: err.to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(data)).into_response()
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

pub async fn get_trading_by_id_handle(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let req = get_trading_controller(id, query).await;
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

pub async fn get_all_tradings_handler(State(query): State<Arc<AppState>>) -> impl IntoResponse {
    let req = get_all_tradings_controllers(query).await;

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
