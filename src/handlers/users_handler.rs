use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{post, get, Route};
use axum::Router;
use axum::{extract::State, Json};
use mongodb::bson::oid::ObjectId;
use mongodb::results::InsertOneResult;

use crate::errors::MyError;
use crate::models::user_model::{self, ModelsController};
use crate::{libs::db::Database, models::user_model::UserModel};

use crate::AppState;

pub async fn create_user_(
    State(app_state) : State<Arc<AppState>>,
    Json(user_fc) : Json<UserModel>
) -> Result<Json<InsertOneResult>, (StatusCode)>{
    let new_user = app_state.db.create_user(user_fc).await;
    match new_user {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_user (
    State(app_state) : State<Arc<AppState>>,
    Path(id) : Path<String>
) -> Result<Json<UserModel> , (StatusCode)>{

    match app_state.db.get_user(&id).await {
        Ok(res) => Ok(res),
        Err(err) => Err(StatusCode::NOT_FOUND)
    }
    // math
    // todo!()
}

pub async fn create_user (
    State(app_state) : State<Arc<AppState>>,
    Json(body) : Json<UserModel>
) -> Result<impl IntoResponse , (StatusCode)>{
    match app_state.db.create_user(body).await {
        Ok(res) => Ok((StatusCode::CREATED)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

pub fn routes(db : Arc<AppState>) -> Router {
    Router::new()
    .route("/", post(create_user))
    .route("/add", post(create_user_))
    .route("/:id", get(get_user))
    .with_state(db)
}