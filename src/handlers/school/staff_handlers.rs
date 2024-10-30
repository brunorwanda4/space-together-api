use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    controller::school::staff_controller::{create_staff_controller, get_staff_by_id_controller},
    error::res_req::ResReq,
    models::school::staff_model::StaffModelNew,
    AppState,
};

pub async fn create_staff_handle(
    State(query): State<Arc<AppState>>,
    Json(staff): Json<StaffModelNew>,
) -> impl IntoResponse {
    let create = create_staff_controller(staff, query).await;
    match create {
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

pub async fn get_staff_by_id_handle(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let get = get_staff_by_id_controller(query, id).await;
    match get {
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
