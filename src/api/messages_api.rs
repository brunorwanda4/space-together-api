use actix_web::{delete, get, post, web, HttpMessage, HttpRequest, HttpResponse};
use mongodb::bson::{doc, oid::ObjectId};
use serde::Deserialize;

use crate::{
    config::state::AppState,
    domain::{
        common_details::Paginated,
        message::{Message, MessageSender, MessageType},
    },
    errors::AppError,
    models::id_model::IdType,
    services::{conversation_service::ConversationService, message_service::MessageService},
};

#[derive(Debug, Deserialize)]
struct CreateMessageRequest {
    encrypted_payload: String,
    nonce: String,
    key_version: Option<i32>,
    message_type: Option<MessageType>,
    file_url: Option<String>,
    file_public_id: Option<String>,
    client_message_id: String,
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<i64>,
    limit: Option<i64>,
}

#[post("/{conversation_id}/messages")]
async fn create_message(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<CreateMessageRequest>,
) -> Result<HttpResponse, AppError> {
    let school_id = req
        .extensions()
        .get::<ObjectId>()
        .copied()
        .ok_or_else(|| AppError { message: "School ID not found".to_string() })?;

    let auth_user = req
        .extensions()
        .get::<crate::domain::auth_user::AuthUserDto>()
        .cloned()
        .ok_or_else(|| AppError { message: "User not authenticated".to_string() })?;

    let conversation_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError { message: "Invalid conversation ID".to_string() })?;

    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let conv_service = ConversationService::new(&db);
    let msg_service = MessageService::new(&db);

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;
    
    let auth_user_role = auth_user.role.clone().unwrap_or(crate::domain::common_details::UserRole::STUDENT);

    let is_participant = conv_service
        .is_participant(conversation_id, auth_user_id)
        .await?;

    if !is_participant {
        return Err(AppError { message: "You are not a participant in this conversation".to_string() });
    }

    let message = Message {
        id: None,
        school_id,
        conversation_id,
        sender: MessageSender {
            sender_role: auth_user_role,
            sender_id: auth_user_id,
        },
        encrypted_payload: body.encrypted_payload.clone(),
        nonce: body.nonce.clone(),
        key_version: body.key_version.unwrap_or(1),
        message_type: body.message_type.clone().unwrap_or(MessageType::TEXT),
        file_url: body.file_url.clone(),
        file_public_id: body.file_public_id.clone(),
        read_by: vec![],
        client_message_id: body.client_message_id.clone(),
        deleted_at: None,
        created_at: chrono::Utc::now(),
    };

    let created = msg_service.create(message).await?;

    Ok(HttpResponse::Created().json(created))
}

#[get("/{conversation_id}/messages")]
async fn get_messages(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let school_id = req
        .extensions()
        .get::<ObjectId>()
        .copied()
        .ok_or_else(|| AppError { message: "School ID not found".to_string() })?;

    let auth_user = req
        .extensions()
        .get::<crate::domain::auth_user::AuthUserDto>()
        .cloned()
        .ok_or_else(|| AppError { message: "User not authenticated".to_string() })?;

    let conversation_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError { message: "Invalid conversation ID".to_string() })?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);

    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let conv_service = ConversationService::new(&db);
    let msg_service = MessageService::new(&db);

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

    let is_participant = conv_service
        .is_participant(conversation_id, auth_user_id)
        .await?;

    if !is_participant {
        return Err(AppError { message: "You are not a participant in this conversation".to_string() });
    }

    let (messages, total) = msg_service
        .get_conversation_messages(conversation_id, page, limit)
        .await?;

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    let response = Paginated {
        data: messages,
        total,
        total_pages,
        current_page: page,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/{conversation_id}/files")]
async fn get_files(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let school_id = req
        .extensions()
        .get::<ObjectId>()
        .copied()
        .ok_or_else(|| AppError { message: "School ID not found".to_string() })?;

    let auth_user = req
        .extensions()
        .get::<crate::domain::auth_user::AuthUserDto>()
        .cloned()
        .ok_or_else(|| AppError { message: "User not authenticated".to_string() })?;

    let conversation_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError { message: "Invalid conversation ID".to_string() })?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let conv_service = ConversationService::new(&db);
    let msg_service = MessageService::new(&db);

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

    let is_participant = conv_service
        .is_participant(conversation_id, auth_user_id)
        .await?;

    if !is_participant {
        return Err(AppError { message: "You are not a participant in this conversation".to_string() });
    }

    let (messages, total) = msg_service
        .get_conversation_files(conversation_id, page, limit)
        .await?;

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    let response = Paginated {
        data: messages,
        total,
        total_pages,
        current_page: page,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[delete("/{conversation_id}/messages/{message_id}")]
async fn delete_message(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let school_id = req
        .extensions()
        .get::<ObjectId>()
        .copied()
        .ok_or_else(|| AppError { message: "School ID not found".to_string() })?;

    let auth_user = req
        .extensions()
        .get::<crate::domain::auth_user::AuthUserDto>()
        .cloned()
        .ok_or_else(|| AppError { message: "User not authenticated".to_string() })?;

    let (conversation_id_str, message_id_str) = path.into_inner();
    let conversation_id = ObjectId::parse_str(&conversation_id_str)
        .map_err(|_| AppError { message: "Invalid conversation ID".to_string() })?;

    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let conv_service = ConversationService::new(&db);
    let msg_service = MessageService::new(&db);

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

    let is_participant = conv_service
        .is_participant(conversation_id, auth_user_id)
        .await?;

    if !is_participant {
        return Err(AppError { message: "You are not a participant in this conversation".to_string() });
    }

    let message = msg_service.find_one(&IdType::String(message_id_str.clone())).await?;

    if message.sender.sender_id != auth_user_id {
        return Err(AppError { message: "You can only delete your own messages".to_string() });
    }

    let deleted = msg_service.soft_delete(&IdType::String(message_id_str)).await?;

    Ok(HttpResponse::Ok().json(deleted))
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create_message)
            .service(get_messages)
            .service(get_files)
            .service(delete_message),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("").configure(blueprint));
}
