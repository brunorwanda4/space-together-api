use std::sync::Arc;

use axum::{routing::post, Router};

use crate::{handlers::class::reason_handler::create_reason_handler, AppState};

pub fn reason_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_reason_handler))
        .with_state(query)
}
