use std::sync::Arc;

use axum::{routing::post, Router};

use crate::{handlers::school::team_handler::create_team_handler, AppState};

pub fn team_routers(query: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_team_handler))
        .with_state(query)
}
