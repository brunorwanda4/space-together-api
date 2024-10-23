use std::sync::Arc;

use axum::Router;

// Import the necessary modules for routes
use super::{
    class::reason_router::reason_routers,
    countries_routes::countries_routes,
    school::{
        school_request_routers::school_request_router, term_routers::term_routers,
        trading_router::trading_routers,
    },
    user_routes::user_routes,
};
use crate::AppState;

// Define all routes
pub async fn all_routes(query: Arc<AppState>) -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .nest("/user", user_routes(Arc::clone(&query)))
            .nest("/countries", countries_routes(Arc::clone(&query)))
            .nest("/schoolRequest", school_request_router(Arc::clone(&query)))
            .nest("/term", term_routers(Arc::clone(&query)))
            .nest("/trading", trading_routers(Arc::clone(&query)))
            .nest("/reason", reason_routers(query)),
    )
}
