use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::class::reason_handler::{create_reason_handler, get_reason_by_id_handler},
    AppState,
};

pub fn reason_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_reason_handler))
        .route("/:id", get(get_reason_by_id_handler))
        .with_state(query)
}
