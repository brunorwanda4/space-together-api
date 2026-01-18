use chrono::Utc;
use futures::channel::mpsc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type EventChannel = mpsc::UnboundedSender<String>;
type EventSessions = Arc<RwLock<HashMap<Uuid, EventChannel>>>;

#[derive(Clone)]
pub struct EventBus {
    sessions: EventSessions,
}

#[derive(Debug, Serialize, Clone)]
pub struct Event {
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<Utc>,
}

impl Event {
    pub fn new(event_type: &str, entity_type: &str, data: serde_json::Value) -> Self {
        Self {
            event_type: event_type.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: None,
            data,
            timestamp: Utc::now(),
        }
    }

    pub fn with_entity_id(mut self, entity_id: &str) -> Self {
        self.entity_id = Some(entity_id.to_string());
        self
    }

    pub fn to_sse_format(&self) -> String {
        let json_data = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        format!("data: {}\n\n", json_data)
    }
}

// Event types
pub const EVENT_CREATED: &str = "created";
pub const EVENT_UPDATED: &str = "updated";
pub const EVENT_DELETED: &str = "deleted";
pub const EVENT_CONNECTED: &str = "connected";

impl EventBus {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new event client
    pub async fn register_client(&self) -> (Uuid, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded();
        let client_id = Uuid::new_v4();

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(client_id, tx);
        }

        println!("🔔 New event client connected: {}", client_id);
        (client_id, rx)
    }

    /// Remove a client
    pub async fn remove_client(&self, client_id: &Uuid) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(client_id);
        println!("🔔 Event client disconnected: {}", client_id);
    }

    /// Broadcast event to all connected clients
    pub async fn broadcast_event(&self, event: &Event) {
        let sessions = self.sessions.read().await;
        let client_count = sessions.len(); // Get count before potential move
        let mut disconnected_clients = Vec::new();

        let message = event.to_sse_format();

        for (client_id, sender) in sessions.iter() {
            if sender.unbounded_send(message.clone()).is_err() {
                disconnected_clients.push(*client_id);
            }
        }

        // Clean up disconnected clients
        if !disconnected_clients.is_empty() {
            drop(sessions); // Explicitly drop the read lock
            let mut sessions_write = self.sessions.write().await;
            for client_id in disconnected_clients {
                sessions_write.remove(&client_id);
            }
        }

        println!(
            "🔔 Broadcasted event: {}::{} to {} clients",
            event.entity_type, event.event_type, client_count
        );
    }

    /// Get number of connected clients
    pub async fn connected_clients_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Helper method to broadcast entity creation
    pub async fn broadcast_created<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_CREATED, entity_type, json_data).with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    /// Helper method to broadcast entity update
    pub async fn broadcast_updated<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_UPDATED, entity_type, json_data).with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    /// Helper method to broadcast entity deletion
    pub async fn broadcast_deleted<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_DELETED, entity_type, json_data).with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
