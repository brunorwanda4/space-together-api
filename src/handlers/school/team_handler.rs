use crate::{
    controller::school::team_controllers::create_team_controller, error::res_req::ResReq,
    models::school::team_model::TeamModelNew, AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn create_team_handler(
    State(db): State<Arc<AppState>>,
    Json(team): Json<TeamModelNew>,
) -> impl IntoResponse {
    let res = create_team_controller(team, None, db).await;

    match res {
        Ok(res) => {
            let data = ResReq {
                success: true,
                message: res,
            };
            (StatusCode::OK, Json(data)).into_response()
        }
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
