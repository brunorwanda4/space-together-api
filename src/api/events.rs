use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web};
use bytes::Bytes;
use futures::StreamExt;

use crate::config::state::AppState;
use crate::domain::auth_user::AuthUserDto;
use crate::models::school_token_model::SchoolToken;
use crate::services::event_bus::{Event, EVENT_CONNECTED};

/// SSE endpoint FOR SCHOOL CONTEXT: /school/events/stream
/// Protected by SchoolTokenMiddleware
#[get("/stream")]
pub async fn school_events_stream(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    // Extract school token from request extensions (set by SchoolTokenMiddleware)
    let school_token = match req.extensions().get::<SchoolToken>() {
        Some(token) => token.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "School token required"
            }));
        }
    };

    let school_id = Some(school_token.id.clone());
let user_id = school_token.member
    .as_ref()
    .and_then(|m| m.get_id())
    .unwrap_or_else(|| "unknown".to_string());

    let (client_id, mut rx) = state
        .event_bus
        .register_client(user_id.clone(), school_id.clone())
        .await;

    let connected_event = Event::new(
        EVENT_CONNECTED,
        "system",
        serde_json::json!({
            "message": "Connected to school event stream",
            "client_id": client_id.to_string(),
            "school_id": school_id,
            "user_id": user_id
        }),
    )
    .for_school(school_id);

    let initial_message = connected_event.to_sse_format();

    let stream = async_stream::stream! {
        yield Ok::<Bytes, actix_web::Error>(Bytes::from(initial_message));

        while let Some(message) = rx.next().await {
            yield Ok::<Bytes, actix_web::Error>(Bytes::from(message));
        }

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

/// SSE endpoint FOR NON-SCHOOL CONTEXT: /events/stream
/// Protected by JwtMiddleware (regular user auth)
#[get("/stream")]
pub async fn global_events_stream(
    _req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let user_id = logged_user.id.clone();

    // No school context - school_id is None
    let (client_id, mut rx) = state
        .event_bus
        .register_client(user_id.clone(), None)
        .await;

    let connected_event = Event::new(
        EVENT_CONNECTED,
        "system",
        serde_json::json!({
            "message": "Connected to global event stream",
            "client_id": client_id.to_string(),
            "user_id": user_id
        }),
    )
    .for_school(None); // Global event

    let initial_message = connected_event.to_sse_format();

    let stream = async_stream::stream! {
        yield Ok::<Bytes, actix_web::Error>(Bytes::from(initial_message));

        while let Some(message) = rx.next().await {
            yield Ok::<Bytes, actix_web::Error>(Bytes::from(message));
        }

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

/// Get connected clients count - school version
#[get("/clients/count")]
pub async fn school_clients_count(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let school_token = match req.extensions().get::<SchoolToken>() {
        Some(token) => token.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "School token required"
            }));
        }
    };

    let count = state
        .event_bus
        .connected_clients_count(Some(&school_token.id))
        .await;

    HttpResponse::Ok().json(serde_json::json!({
        "connected_clients": count,
        "school_id": school_token.id
    }))
}

/// Get connected clients count - global version
#[get("/clients/count")]
pub async fn global_clients_count(state: web::Data<AppState>) -> impl Responder {
    let count = state.event_bus.connected_clients_count(None).await;

    HttpResponse::Ok().json(serde_json::json!({
        "connected_clients": count
    }))
}

/// Initialize event routes using dual mount pattern
pub fn init(cfg: &mut web::ServiceConfig) {
    // School events endpoint - wrapped with SchoolTokenMiddleware
    cfg.service(
        web::scope("/school/events")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            .service(school_events_stream)
            .service(school_clients_count),
    );

    // Global events endpoint - wrapped with JwtMiddleware
    cfg.service(
        web::scope("/events")
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(global_events_stream)
            .service(global_clients_count),
    );
}