use crate::{
    controller::school::term_controllers::{
        create_term_controller, get_term_controller, update_term_controller,
    },
    error::res_req::ResReq,
    models::school::term_model::{TermModelNew, TermModelUpdate},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn create_term_handler(
    State(db): State<Arc<AppState>>,
    Json(term): Json<TermModelNew>,
) -> impl IntoResponse {
    let res = create_term_controller(term, None, db).await;

    match res {
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

pub async fn get_term_handle(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = get_term_controller(query, id).await;

    match res {
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

pub async fn update_term_handle(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(term): Json<TermModelUpdate>,
) -> impl IntoResponse {
    let res = update_term_controller(query, term, id).await;
    match res {
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
