use std::sync::Arc;

use axum::{
    routing::{post, put},
    Router,
};

use crate::{
    handlers::school::trading_handler::{create_trading_handler, update_trading_handle},
    AppState,
};

pub fn trading_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_trading_handler))
        .route("/:id", put(update_trading_handle))
        .with_state(query)
}
