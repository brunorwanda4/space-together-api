use chrono::Utc;
use futures::channel::mpsc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type EventChannel = mpsc::UnboundedSender<String>;

// Client session now tracks which school's database they're listening to
#[derive(Clone, Debug)]
pub struct ClientSession {
    pub sender: EventChannel,
    pub database_name: String, // The school's database name
    pub user_id: Option<String>, // Optional for user-specific events
}

type EventSessions = Arc<RwLock<HashMap<Uuid, ClientSession>>>;

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
    // Scope metadata
    pub database_name: String, // Which school's database this event belongs to
    pub user_id: Option<String>, // Optional: for user-specific events
}

impl Event {
    pub fn new(
        event_type: &str,
        entity_type: &str,
        database_name: &str,
        data: serde_json::Value,
    ) -> Self {
        Self {
            event_type: event_type.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: None,
            data,
            timestamp: Utc::now(),
            database_name: database_name.to_string(),
            user_id: None,
        }
    }

    pub fn with_entity_id(mut self, entity_id: &str) -> Self {
        self.entity_id = Some(entity_id.to_string());
        self
    }

    pub fn for_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn to_sse_format(&self) -> String {
        let json_data = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        format!("data: {}\n\n", json_data)
    }

    // Check if event should be sent to a specific client
    fn should_send_to_client(&self, client: &ClientSession) -> bool {
        // First check: must be same database (school)
        if self.database_name != client.database_name {
            return false;
        }

        // Second check: if event is user-specific, must match user_id
        if let Some(event_user_id) = &self.user_id {
            if let Some(client_user_id) = &client.user_id {
                return event_user_id == client_user_id;
            }
            return false; // Event is for a specific user, but client has no user_id
        }

        // Event is for the whole school and client is from that school
        true
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

    /// Register a new event client with database context
    pub async fn register_client(
        &self,
        database_name: String,
        user_id: Option<String>,
    ) -> (Uuid, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded();
        let client_id = Uuid::new_v4();

        let session = ClientSession {
            sender: tx,
            database_name: database_name.clone(),
            user_id: user_id.clone(),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(client_id, session);
        }

        println!(
            "🔔 New event client connected: {} (db: {}, user: {:?})",
            client_id, database_name, user_id
        );
        (client_id, rx)
    }

    /// Remove a client
    pub async fn remove_client(&self, client_id: &Uuid) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(client_id);
        println!("🔔 Event client disconnected: {}", client_id);
    }

    /// Broadcast event to filtered clients based on database and user
    pub async fn broadcast_event(&self, event: &Event) {
        let sessions = self.sessions.read().await;
        let mut disconnected_clients = Vec::new();
        let mut sent_count = 0;

        let message = event.to_sse_format();

        for (client_id, client_session) in sessions.iter() {
            // Check if this event should be sent to this client
            if event.should_send_to_client(client_session) {
                if client_session.sender.unbounded_send(message.clone()).is_err() {
                    disconnected_clients.push(*client_id);
                } else {
                    sent_count += 1;
                }
            }
        }

        // Clean up disconnected clients
        if !disconnected_clients.is_empty() {
            drop(sessions.clone());
            let mut sessions_write = self.sessions.write().await;
            for client_id in disconnected_clients {
                sessions_write.remove(&client_id);
            }
        }

        println!(
            "🔔 Broadcasted event: {}::{} to {}/{} clients (db: {}, user: {:?})",
            event.entity_type,
            event.event_type,
            sent_count,
            sessions.len(),
            event.database_name,
            event.user_id
        );
    }

    /// Get number of connected clients (optionally filtered by database)
    pub async fn connected_clients_count(&self, database_name: Option<&str>) -> usize {
        let sessions = self.sessions.read().await;
        if let Some(db) = database_name {
            sessions
                .values()
                .filter(|s| s.database_name == db)
                .count()
        } else {
            sessions.len()
        }
    }

    /// Helper method to broadcast entity creation
    pub async fn broadcast_created<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        database_name: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_CREATED, entity_type, database_name, json_data)
            .with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    /// Helper method to broadcast entity update
    pub async fn broadcast_updated<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        database_name: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_UPDATED, entity_type, database_name, json_data)
            .with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    /// Helper method to broadcast entity deletion
    pub async fn broadcast_deleted<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        database_name: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_DELETED, entity_type, database_name, json_data)
            .with_entity_id(entity_id);
        self.broadcast_event(&event).await;
    }

    /// Helper method to broadcast user-specific event
    pub async fn broadcast_to_user<T: Serialize>(
        &self,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        database_name: &str,
        user_id: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(event_type, entity_type, database_name, json_data)
            .with_entity_id(entity_id)
            .for_user(user_id);
        self.broadcast_event(&event).await;
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}