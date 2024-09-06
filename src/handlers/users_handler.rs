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
use crate::models::user_model::{self, ModelsController, UpdateUserModel};
use crate::{libs::db::Database, models::user_model::UserModel};

use crate::AppState;

pub async fn create_user(
    State(app_state) : State<Arc<AppState>>,
    Json(user_fc) : Json<UserModel>
) -> Result<Json<InsertOneResult>, (StatusCode)>{
    let user_email = user_fc.email.clone();
    let find_user_email = app_state.db.get_user_by_email(user_email);
    
    if find_user_email.await.is_ok() {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }
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
}

pub async fn update_user (
    Path(id) : Path<String>,
    State(app_state) : State<Arc<AppState>>,
    Json(user_fc) : Json<UpdateUserModel>
) -> Result<Json <UserModel> , (StatusCode)>{
    // let find_user = app_state.db.get_user(&id).await;

    // if find_user.is_err() {
    //     return Err(StatusCode::NOT_FOUND);
    // }
    let res = app_state.db.update_user(&id, &user_fc).await;

    match res {
        Ok(user) => Ok(Json(user)),
        Err(status) => Err(StatusCode::BAD_REQUEST)
    }
}
