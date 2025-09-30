use crate::services::event_bus::EventBus;
use mongodb::Database;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub event_bus: Arc<EventBus>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            event_bus: Arc::new(EventBus::new()),
        }
    }
}
