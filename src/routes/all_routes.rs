use std::sync::Arc;

use axum::{routing::{get, get_service, post, put}, Router};
use tower_http::services::ServeDir;

use crate::AppState;

use super::{countries_routes::countries_routes, school::school_request_routers::school_request_router, user_routes::user_routes};
pub async fn all_routes (query : Arc<AppState>) -> Router{    
    Router::new()
    .nest("/api/v1/user", user_routes(query.clone()))
    .nest("/api/v1/countries", countries_routes(query.clone()))
    .nest("/api/v1/schoolRequest", school_request_router(query))
}