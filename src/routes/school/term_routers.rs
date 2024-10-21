use std::sync::Arc;

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{
    handlers::school::term_handler::{create_term_handler, get_term_handle, update_term_handle},
    AppState,
};

pub fn term_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_term_handler))
        .route("/:id", get(get_term_handle))
        .route("/:id", put(update_term_handle))
        .with_state(query)
}
