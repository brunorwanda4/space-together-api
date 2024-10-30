use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::class::reason_handler::{
        create_reason_handler, delete_reason_by_id_handle, get_reason_by_id_handler,
        update_reason_by_id_handle,
    },
    AppState,
};

pub fn reason_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_reason_handler))
        .route("/:id", get(get_reason_by_id_handler))
        .route("/:id", put(update_reason_by_id_handle))
        .route("/:id", delete(delete_reason_by_id_handle))
        .with_state(query)
}
