use crate::config::mongo_manager::MongoManager;
use crate::config::postgres_manager::PgManager;
use crate::services::event_bus::EventBus;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: MongoManager, // new
    #[allow(dead_code)]
    pub pg: PgManager,
    pub event_bus: Arc<EventBus>,
}

impl AppState {
    pub fn new(db: MongoManager, pg: PgManager) -> Self {
        Self {
            db,
            pg,
            event_bus: Arc::new(EventBus::new()),
        }
    }
}
