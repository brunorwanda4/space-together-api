use std::sync::Arc;

use axum::{routing::{get, post}, Router};

use crate::{
    handlers::users_handler::{
        self, create_user, get_user
    }, AppState};

pub fn user_routes(db : Arc<AppState>) -> Router {
    Router::new()
    .route("/", post(create_user))
    .route("/:id", get(get_user))
    .with_state(db)
}

pub async fn all_routers (app_state : Arc<AppState>) -> Router{    
    Router::new()
    .nest("/api/v1/user", user_routes(app_state))
}