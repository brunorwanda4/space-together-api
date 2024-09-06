use std::sync::Arc;

use axum::{routing::{get, get_service, post, put}, Router};
use tower_http::services::ServeDir;

use crate::{
    handlers::{user_auth_handler::user_login_handler, users_handler::{
        self, create_user, get_user, update_user
    }}, AppState};

pub fn user_routes(db : Arc<AppState>) -> Router {
    Router::new()
    .route("/", post(create_user))
    // .route("/auth/login", post(user_login_handler))
    .route(
        "/:id",
        get(get_user)
        .put(update_user))
    .with_state(db)
}

pub async fn all_routers (app_state : Arc<AppState>) -> Router{    
    Router::new()
    .nest("/api/v1/user", user_routes(app_state))
}