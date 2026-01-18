use chrono::Utc;
use futures::channel::mpsc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type EventChannel = mpsc::UnboundedSender<String>;

#[derive(Clone, Debug)]
pub struct ClientSession {
    pub sender: EventChannel,
    pub school_id: Option<String>, // Optional for non-school users
    pub user_id: String,
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
    pub school_id: Option<String>, // Which school this event belongs to
    pub target_user_id: Option<String>, // For user-specific events
}

impl Event {
    pub fn new(event_type: &str, entity_type: &str, data: serde_json::Value) -> Self {
        Self {
            event_type: event_type.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: None,
            data,
            timestamp: Utc::now(),
            school_id: None,
            target_user_id: None,
        }
    }

    pub fn with_entity_id(mut self, entity_id: &str) -> Self {
        self.entity_id = Some(entity_id.to_string());
        self
    }

    pub fn for_school(mut self, school_id: Option<String>) -> Self {
        self.school_id = school_id;
        self
    }

    pub fn for_user(mut self, user_id: &str) -> Self {
        self.target_user_id = Some(user_id.to_string());
        self
    }

    pub fn to_sse_format(&self) -> String {
        let json_data = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        format!("data: {}\n\n", json_data)
    }

    /// Check if event should be sent to a specific client
    fn should_send_to_client(&self, client: &ClientSession) -> bool {
        // User-specific event - only send to that user
        if let Some(target_user) = &self.target_user_id {
            return target_user == &client.user_id;
        }

        // School event logic
        match (&self.school_id, &client.school_id) {
            // Event has school_id and client has school_id - must match
            (Some(event_school), Some(client_school)) => event_school == client_school,
            
            // Event has no school_id and client has no school_id - send it (global events)
            (None, None) => true,
            
            // Mismatch: event is for school but client is not in a school, or vice versa
            _ => false,
        }
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

    /// Register a new event client (school_id is optional)
    pub async fn register_client(
        &self,
        user_id: String,
        school_id: Option<String>,
    ) -> (Uuid, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded();
        let client_id = Uuid::new_v4();

        let session = ClientSession {
            sender: tx,
            school_id: school_id.clone(),
            user_id: user_id.clone(),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(client_id, session);
        }

        println!(
            "🔔 New event client connected: {} (user: {}, school: {:?})",
            client_id, user_id, school_id
        );
        (client_id, rx)
    }

    /// Remove a client
    pub async fn remove_client(&self, client_id: &Uuid) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(client_id);
        println!("🔔 Event client disconnected: {}", client_id);
    }

    /// Broadcast event with automatic filtering
    pub async fn broadcast_event(&self, event: &Event) {
        let sessions = self.sessions.read().await;
        let mut disconnected_clients = Vec::new();
        let mut sent_count = 0;

        let message = event.to_sse_format();

        for (client_id, client_session) in sessions.iter() {
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
            "🔔 Broadcasted event: {}::{} to {}/{} clients (school: {:?})",
            event.entity_type,
            event.event_type,
            sent_count,
            sessions.len(),
            event.school_id
        );
    }

    /// Get number of connected clients
    pub async fn connected_clients_count(&self, school_id: Option<&str>) -> usize {
        let sessions = self.sessions.read().await;
        if let Some(school) = school_id {
            sessions
                .values()
                .filter(|s| s.school_id.as_deref() == Some(school))
                .count()
        } else {
            sessions
                .values()
                .filter(|s| s.school_id.is_none())
                .count()
        }
    }

    /// Helper: broadcast entity creation
    pub async fn broadcast_created<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        school_id: Option<String>,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_CREATED, entity_type, json_data)
            .with_entity_id(entity_id)
            .for_school(school_id);
        self.broadcast_event(&event).await;
    }

    /// Helper: broadcast entity update
    pub async fn broadcast_updated<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        school_id: Option<String>,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_UPDATED, entity_type, json_data)
            .with_entity_id(entity_id)
            .for_school(school_id);
        self.broadcast_event(&event).await;
    }

    /// Helper: broadcast entity deletion
    pub async fn broadcast_deleted<T: Serialize>(
        &self,
        entity_type: &str,
        entity_id: &str,
        school_id: Option<String>,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(EVENT_DELETED, entity_type, json_data)
            .with_entity_id(entity_id)
            .for_school(school_id);
        self.broadcast_event(&event).await;
    }

    /// Helper: broadcast to specific user
    pub async fn broadcast_to_user<T: Serialize>(
        &self,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        user_id: &str,
        data: &T,
    ) {
        let json_data = serde_json::to_value(data).unwrap_or_else(|_| serde_json::Value::Null);
        let event = Event::new(event_type, entity_type, json_data)
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