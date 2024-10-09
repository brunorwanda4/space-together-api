use std::sync::Arc;

use axum::{routing::{post, Route}, Router};

use crate::{handlers::country_handler::add_country, AppState};

pub fn countries_routes(db : Arc<AppState>) -> Router {
    Router::new()
        .route( "/", post(add_country))
        .with_state(db)
}