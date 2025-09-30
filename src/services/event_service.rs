use crate::config::state::AppState;
use actix_web::web;
use serde::Serialize;

pub struct EventService;

impl EventService {
    /// Broadcast entity creation event
    pub async fn broadcast_created<T: Serialize>(
        state: &web::Data<AppState>,
        entity_type: &str,
        entity_id: &str,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_created(entity_type, entity_id, data)
            .await;
    }

    /// Broadcast entity update event
    pub async fn broadcast_updated<T: Serialize>(
        state: &web::Data<AppState>,
        entity_type: &str,
        entity_id: &str,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_updated(entity_type, entity_id, data)
            .await;
    }

    /// Broadcast entity deletion event
    pub async fn broadcast_deleted<T: Serialize>(
        state: &web::Data<AppState>,
        entity_type: &str,
        entity_id: &str,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_deleted(entity_type, entity_id, data)
            .await;
    }

    /// Get connected clients count
    pub async fn connected_clients_count(state: &web::Data<AppState>) -> usize {
        state.event_bus.connected_clients_count().await
    }
}
