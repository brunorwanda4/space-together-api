use std::sync::Arc;

use axum::{routing::{post , get}, Router};

use crate::{handlers::school::school_request_handlers::{create_school_request_handlers, get_school_request_handler}, AppState};

pub fn school_request_router (query : Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_school_request_handlers))
        .route("/:id", get(get_school_request_handler))
        .with_state(query)
}