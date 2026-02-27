use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    config::state::AppState,
    domain::{
        common_details::{Paginated, RelatedUser, UserRole},
        conversation::{Conversation, ConversationKey},
    },
    errors::AppError,
    models::id_model::IdType,
    services::conversation_service::ConversationService,
};

#[derive(Debug, Deserialize)]
struct CreateConversationRequest {
    participants: Vec<RelatedUser>,
    is_group: bool,
    name: Option<String>,
    encrypted_keys: Vec<EncryptedKeyForUser>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedKeyForUser {
    user_id: String,
    user_role: UserRole,
    encrypted_key: String,
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<i64>,
    limit: Option<i64>,
}

#[post("")]
async fn create_conversation(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateConversationRequest>,
) -> Result<impl Responder, AppError> {
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

    // Validate participants
    if body.is_group && body.participants.len() < 3 {
        return Err(AppError { message: "Group conversations require at least 3 participants".to_string() });
    }
    if !body.is_group && body.participants.len() != 2 {
        return Err(AppError { message: "Direct conversations require exactly 2 participants".to_string() });
    }

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;
    
    let user_in_participants = body.participants.iter().any(|p| {
        p.get_id().map(|id| id == auth_user.id).unwrap_or(false)
    });

    if !user_in_participants {
        return Err(AppError { message: "You must be a participant in the conversation".to_string() });
    }

    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let service = ConversationService::new(&db);

    let conversation = Conversation {
        id: None,
        school_id,
        participants: body.participants.clone(),
        is_group: body.is_group,
        name: body.name.clone(),
        encryption_key_version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = service.create(conversation).await?;

    // Store encrypted keys for each participant
    for key_data in &body.encrypted_keys {
        let user_id = ObjectId::parse_str(&key_data.user_id)
            .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

        let key = ConversationKey {
            id: None,
            conversation_id: created.id.unwrap(),
            user_id,
            user_role: key_data.user_role.clone(),
            encrypted_key_for_user: key_data.encrypted_key.clone(),
            created_at: chrono::Utc::now(),
        };

        service.store_conversation_key(key).await?;
    }

    Ok(HttpResponse::Created().json(created))
}

#[get("")]
async fn get_conversations(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<QueryParams>,
) -> Result<impl Responder, AppError> {
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

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let service = ConversationService::new(&db);

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

    let filter = doc! {
        "school_id": school_id,
        "participants.user.id": auth_user_id
    };

    let (conversations, total) = service.get_all(filter, page, limit).await?;

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    let response = Paginated {
        data: conversations,
        total,
        total_pages,
        current_page: page,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}")]
async fn get_conversation(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
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

    let id = path.into_inner();
    let db = state.db.get_db(&format!("school_{}", school_id.to_hex()));
    let service = ConversationService::new(&db);

    let conversation = service.find_one(&IdType::String(id.clone())).await?;

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

    let is_participant = service
        .is_participant(conversation.id.unwrap(), auth_user_id)
        .await?;

    if !is_participant {
        return Err(AppError { message: "You are not a participant in this conversation".to_string() });
    }

    Ok(HttpResponse::Ok().json(conversation))
}

#[get("/{id}/key")]
async fn get_conversation_key(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
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
    let service = ConversationService::new(&db);

    let auth_user_id = ObjectId::parse_str(&auth_user.id)
        .map_err(|_| AppError { message: "Invalid user ID".to_string() })?;

    let is_participant = service
        .is_participant(conversation_id, auth_user_id)
        .await?;

    if !is_participant {
        return Err(AppError { message: "You are not a participant in this conversation".to_string() });
    }

    let key = service.get_conversation_key(conversation_id, auth_user_id).await?;

    Ok(HttpResponse::Ok().json(key))
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create_conversation)
            .service(get_conversations)
            .service(get_conversation)
            .service(get_conversation_key),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/conversations").configure(blueprint));
}
