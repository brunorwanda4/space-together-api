use actix_web::{get, web, Error, HttpMessage, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::StreamExt;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::config::state::AppState;

// WebSocket message types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsMessage {
    MessageCreated { conversation_id: String, message_id: String },
    MessageRead { message_id: String, user_id: String },
    MessageDeleted { message_id: String },
    ConversationCreated { conversation_id: String },
    ParticipantAdded { conversation_id: String, user_id: String },
    Error { message: String },
    Ping,
    Pong,
}

#[get("/m/ws/{conversation_id}")]
async fn websocket_handler(
    req: HttpRequest,
    path: web::Path<String>,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Extract authentication from request
    let school_id = req
        .extensions()
        .get::<ObjectId>()
        .copied()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("School ID not found"))?;

    let auth_user = req
        .extensions()
        .get::<crate::domain::auth_user::AuthUserDto>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?;

    let conversation_id = path.into_inner();

    // Verify user is participant in conversation
    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    
    let conv_service = crate::services::conversation_service::ConversationService::new(&db);
    let conversation_oid = ObjectId::parse_str(&conversation_id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid conversation ID"))?;

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;

    let is_participant = conv_service
        .is_participant(conversation_oid, auth_user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if !is_participant {
        return Err(actix_web::error::ErrorForbidden("Not a participant"));
    }

    let (response, mut session, mut stream) = actix_ws::handle(&req, stream)?;

    // Spawn task to handle WebSocket messages
    actix_web::rt::spawn(async move {
        log::info!("WebSocket connection established for conversation: {}", conversation_id);

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    log::debug!("Received text message: {}", text);
                    
                    // Parse incoming message
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Ping => {
                                let pong = serde_json::to_string(&WsMessage::Pong).unwrap();
                                let _ = session.text(pong).await;
                            }
                            _ => {
                                log::warn!("Unexpected message type from client");
                            }
                        }
                    }
                }
                Ok(Message::Ping(bytes)) => {
                    let _ = session.pong(&bytes).await;
                }
                Ok(Message::Close(reason)) => {
                    let _ = session.close(reason).await;
                    break;
                }
                _ => break,
            }
        }

        log::info!("WebSocket connection closed for conversation: {}", conversation_id);
    });

    Ok(response)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(websocket_handler);
}
