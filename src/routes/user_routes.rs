use std::sync::Arc;

use axum::{routing::{get, post}, Router};

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