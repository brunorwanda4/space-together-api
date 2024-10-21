use std::sync::Arc;

use axum::Router;

use crate::AppState;

use super::{
    countries_routes::countries_routes,
    school::{
        school_request_routers::school_request_router, term_routers::term_routers,
        trading_router::trading_routers,
    },
    user_routes::user_routes,
};
pub async fn all_routes(query: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1/user", user_routes(query.clone()))
        .nest("/api/v1/countries", countries_routes(query.clone()))
        .nest(
            "/api/v1/schoolRequest",
            school_request_router(query.clone()),
        )
        .nest("/api/v1/term", term_routers(query.clone()))
        .nest("/api/v1/trading", trading_routers(query))
}
