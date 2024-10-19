use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    controller::school::trading_controller::create_trading_controller, error::res_req::ResReq,
    models::school::trading_model::TradingModelNew, AppState,
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
            (StatusCode::BAD_REQUEST, Json(data)).into_response()
        }
    }
}
