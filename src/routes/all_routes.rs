use std::sync::Arc;

use axum::Router;

// Import the necessary modules for routes
use super::{
    class::reason_router::reason_routers,
    countries_routes::countries_routes,
    school::{
        school_request_routers::school_request_router, staff_routers::staff_routers,
        term_routers::term_routers, trading_router::trading_routers,
    },
    user_routes::user_routes,
};
use crate::AppState;

// Define all routes
pub async fn all_routes(query: Arc<AppState>) -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .nest("/user", user_routes(query.clone()))
            .nest("/countries", countries_routes(query.clone()))
            .nest("/term", term_routers(query.clone()))
            .nest("/schoolRequest", school_request_router(query.clone()))
            .nest("/trading", trading_routers(query.clone()))
            .nest("/reason", reason_routers(query.clone()))
            .nest("/school-staff", staff_routers(query)),
    )
}
