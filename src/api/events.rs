use actix_web::{get, web, HttpResponse, Responder};
use bytes::Bytes;
use futures::StreamExt;

use crate::config::state::AppState;
use crate::services::event_bus::{Event, EVENT_CONNECTED};

/// SSE endpoint for real-time events - KEEP THIS SEPARATE
#[get("/stream")]
pub async fn events_stream(state: web::Data<AppState>) -> impl Responder {
    let (client_id, mut rx) = state.event_bus.register_client().await;

    // Create initial connection event
    let connected_event = Event::new(
        EVENT_CONNECTED,
        "system",
        serde_json::json!({
            "message": "Connected to real-time event stream",
            "client_id": client_id.to_string()
        }),
    );

    let initial_message = connected_event.to_sse_format();

    // Create stream from receiver
    let stream = async_stream::stream! {
        // Send initial connection event
        yield Ok::<Bytes, actix_web::Error>(Bytes::from(initial_message));

        // Handle incoming messages
        while let Some(message) = rx.next().await {
            yield Ok::<Bytes, actix_web::Error>(Bytes::from(message));
        }

        // Client disconnected - cleanup
        let event_bus = state.event_bus.clone();
        actix_web::rt::spawn(async move {
            event_bus.remove_client(&client_id).await;
        });
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Credentials", "true"))
        .streaming(stream)
}

/// Get connected clients count (admin only)
#[get("/stream/clients/count")]
pub async fn get_connected_clients_count(state: web::Data<AppState>) -> impl Responder {
    let count = state.event_bus.connected_clients_count().await;
    HttpResponse::Ok().json(serde_json::json!({
        "connected_clients": count
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/events")
            .service(events_stream)
            .service(get_connected_clients_count),
    );
}
