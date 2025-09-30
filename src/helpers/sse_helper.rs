use crate::models::sse_model::{SSEEvent, EVENT_CREATED, EVENT_DELETED, EVENT_UPDATED};
use crate::services::sse_service::SSEService;
use actix_web::web;
use serde::Serialize;

#[async_trait::async_trait]
pub trait SSEHelper {
    async fn broadcast_created<T: Serialize>(&self, entity_type: &str, entity_id: &str, data: &T);
    async fn broadcast_updated<T: Serialize>(&self, entity_type: &str, entity_id: &str, data: &T);
    async fn broadcast_deleted<T: Serialize>(&self, entity_type: &str, entity_id: &str, data: &T);
    async fn broadcast_custom(&self, event_type: &str, entity_type: &str, data: serde_json::Value);
}

#[async_trait::async_trait]
impl SSEHelper for web::Data<SSEService> {
    async fn broadcast_created<T: Serialize>(&self, entity_type: &str, entity_id: &str, data: &T) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = SSEEvent::new(EVENT_CREATED, entity_type, json_data).with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    async fn broadcast_updated<T: Serialize>(&self, entity_type: &str, entity_id: &str, data: &T) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = SSEEvent::new(EVENT_UPDATED, entity_type, json_data).with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    async fn broadcast_deleted<T: Serialize>(&self, entity_type: &str, entity_id: &str, data: &T) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = SSEEvent::new(EVENT_DELETED, entity_type, json_data).with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    async fn broadcast_custom(&self, event_type: &str, entity_type: &str, data: serde_json::Value) {
        let event = SSEEvent::new(event_type, entity_type, data);
        self.broadcast_event(&event).await;
    }
}
