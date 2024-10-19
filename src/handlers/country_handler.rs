use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    error::res_req::ResReq,
    errors::{MyError, Result},
    models::country_model::CountryModel,
    AppState,
};

pub async fn add_country(
    State(app_state): State<Arc<AppState>>,
    Json(country): Json<CountryModel>,
) -> impl IntoResponse {
    let res = app_state
        .db
        .country_action_db
        .create_country(&country)
        .await;

    match res {
        Ok(insert_res) => (StatusCode::OK, Json(insert_res)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };

            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}
