use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::school::team_handler::{create_team_handler, get_team_handle},
    AppState,
};

pub fn team_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_team_handler))
        .route("/:id", get(get_team_handle))
        .with_state(query)
}
