use crate::config::state::AppState;
use actix_web::web;
use serde::Serialize;

pub struct EventService;

impl EventService {
    /// Broadcast entity creation event
    /// school_id: Some(id) for school events, None for global events
    pub async fn broadcast_created<T: Serialize>(
        state: &web::Data<AppState>,
        entity_type: &str,
        entity_id: &str,
        school_id: Option<String>,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_created(entity_type, entity_id, school_id, data)
            .await;
    }

    /// Broadcast entity update event
    pub async fn broadcast_updated<T: Serialize>(
        state: &web::Data<AppState>,
        entity_type: &str,
        entity_id: &str,
        school_id: Option<String>,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_updated(entity_type, entity_id, school_id, data)
            .await;
    }

    /// Broadcast entity deletion event
    pub async fn broadcast_deleted<T: Serialize>(
        state: &web::Data<AppState>,
        entity_type: &str,
        entity_id: &str,
        school_id: Option<String>,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_deleted(entity_type, entity_id, school_id, data)
            .await;
    }

    /// Broadcast to specific user (e.g., notifications)
    pub async fn broadcast_to_user<T: Serialize>(
        state: &web::Data<AppState>,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        user_id: &str,
        data: &T,
    ) {
        state
            .event_bus
            .broadcast_to_user(event_type, entity_type, entity_id, user_id, data)
            .await;
    }
}
