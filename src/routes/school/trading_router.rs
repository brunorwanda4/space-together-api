use std::sync::Arc;

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{
    handlers::school::trading_handler::{
        create_trading_handler, get_all_tradings_handler, get_trading_by_id_handle,
        update_trading_handle,
    },
    AppState,
};

pub fn trading_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_trading_handler))
        .route("/", get(get_all_tradings_handler))
        .route("/:id", put(update_trading_handle))
        .route("/:id", get(get_trading_by_id_handle))
        .with_state(query)
}
