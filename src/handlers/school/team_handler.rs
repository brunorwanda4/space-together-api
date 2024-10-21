use crate::{
    controller::school::team_controllers::{create_team_controller, get_team_controller},
    error::res_req::ResReq,
    models::school::team_model::TeamModelNew,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn create_team_handler(
    State(db): State<Arc<AppState>>,
    Json(team): Json<TeamModelNew>,
) -> impl IntoResponse {
    let res = create_team_controller(team, None, db).await;

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

pub async fn get_team_handle(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = get_team_controller(query, id).await;

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
