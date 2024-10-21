use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    controller::class::reason_controller::{create_reason_controller, get_reason_controller},
    error::res_req::ResReq,
    models::class::reasons_model::ReasonModelNew,
    AppState,
};

pub async fn create_reason_handler(
    State(query): State<Arc<AppState>>,
    Json(reason): Json<ReasonModelNew>,
) -> impl IntoResponse {
    let res = create_reason_controller(query, reason).await;
    match res {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

pub async fn get_reason_by_id_handler(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = get_reason_controller(query, id).await;
    match res {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
