use std::sync::Arc;

use axum::{routing::post, Router};

use crate::{handlers::school::trading_handler::create_trading_handler, AppState};

pub fn trading_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_trading_handler))
        .with_state(query)
}
