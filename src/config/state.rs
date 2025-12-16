use crate::config::mongo_manager::MongoManager;
use crate::services::event_bus::EventBus;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: MongoManager, // new
    pub event_bus: Arc<EventBus>,
}

impl AppState {
    pub fn new(db: MongoManager) -> Self {
        Self {
            db,
            event_bus: Arc::new(EventBus::new()),
        }
    }
}
