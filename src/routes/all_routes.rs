use std::sync::Arc;

use axum::{routing::{get, get_service, post, put}, Router};
use tower_http::services::ServeDir;

use crate::AppState;

use super::user_routes::user_routes;


pub async fn all_routes (app_state : Arc<AppState>) -> Router{    
    Router::new()
    .nest("/api/v1/user", user_routes(app_state))
}