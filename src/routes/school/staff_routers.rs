use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::school::staff_handlers::{create_staff_handle, get_staff_by_id_handle},
    AppState,
};

pub fn staff_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_staff_handle))
        .route("/:id", get(get_staff_by_id_handle))
        .with_state(query)
}
