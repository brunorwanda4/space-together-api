use std::sync::Arc;

    use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{post, get, Route};
use axum::Router;
use axum::{extract::State, Json};
use mongodb::bson::oid::ObjectId;
use mongodb::results::InsertOneResult;
use serde::{Deserialize, Serialize};

use crate::errors::MyError;
use crate::models::user_model::{self, CreateUserRequestModel, ModelsController, UpdateUserModel};
use crate::{libs::db::Database, models::user_model::UserModel};

use crate::AppState;

#[derive(Debug , Deserialize ,Serialize)]
pub struct CreateUserResultError {
    success: bool,
    message: String,
}

pub async fn create_user(
    State(app_state): State<Arc<AppState>>,
    Json(user_fc): Json<CreateUserRequestModel>,
) -> impl IntoResponse {
    let user_email = user_fc.email.clone();
    let find_user_email = app_state.db.get_user_by_email(user_email.clone()).await;

    if find_user_email.is_ok() {
        let error_response = CreateUserResultError {
            success: false,
            message: MyError::UserEmailIsReadyExit { email: user_email }.to_string(),
        };
        return (StatusCode::NOT_ACCEPTABLE, Json(error_response)).into_response();
    }

    let new_user = app_state.db.create_user(user_fc.name, user_fc.email, Some(user_fc.password)).await;
    match new_user {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(_) => {
            let error_response = CreateUserResultError {
                success: false,
                message: MyError::CreateUserError.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}


pub async fn get_user(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user = app_state.db.get_user(&id).await;

    match user {
        Ok(user) => Ok(user),
        Err(err) => {
            let error_response = CreateUserResultError {
                success: false,
                message: err.to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response())
        }
    }
}

pub async fn update_user (
    Path(id) : Path<String>,
    State(app_state) : State<Arc<AppState>>,
    Json(user_fc) : Json<UpdateUserModel>
) -> Result<Json <UserModel> , (StatusCode)>{
    let res = app_state.db.update_user(&id, &user_fc).await;

    match res {
        Ok(user) => Ok(Json(user)),
        Err(status) => Err(StatusCode::BAD_REQUEST)
    }
}
