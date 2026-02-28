use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    config::state::AppState, domain::{
        auth_user::AuthUserDto,
        common_details::{RelatedUser, UserRole},
        conversation::{Conversation, ConversationKey},
    }, errors::AppError, middleware::school_token_middleware::SchoolTokenMiddleware, models::{id_model::IdType, school_token_model::SchoolToken}, services::conversation_service::ConversationService, utils::db_utils::get_database
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
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    body: web::Json<CreateConversationRequest>,
) -> impl Responder {
    let auth_user = user.into_inner();

    // School ID is optional - if present, conversation is school-specific
    // If not present, conversation is stored in main database (cross-school or admin)
    let school_id = req.extensions().get::<ObjectId>().copied();

    // Validate participants
    if body.is_group && body.participants.len() < 3 {
        return HttpResponse::BadRequest().json(AppError { 
            message: "Group conversations require at least 3 participants".to_string() 
        });
    }
    if !body.is_group && body.participants.len() != 2 {
        return HttpResponse::BadRequest().json(AppError { 
            message: "Direct conversations require exactly 2 participants".to_string() 
        });
    }

    let user_in_participants = body.participants.iter().any(|p| {
        p.get_id().map(|id| id == auth_user.id).unwrap_or(false)
    });

    if !user_in_participants {
        return HttpResponse::BadRequest().json(AppError { 
            message: "You must be a participant in the conversation".to_string() 
        });
    }

    let db = get_database(&req, &state);
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

    let created = match service.create(conversation).await {
        Ok(conv) => conv,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let conversation_id = created.id.unwrap();

    // Store encrypted keys for each participant
    for key_data in &body.encrypted_keys {
        let user_id = match ObjectId::parse_str(&key_data.user_id) {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().json(AppError { 
                message: "Invalid user ID".to_string() 
            }),
        };

        let key = ConversationKey {
            id: None,
            conversation_id,
            user_id,
            user_role: key_data.user_role.clone(),
            encrypted_key_for_user: key_data.encrypted_key.clone(),
            created_at: chrono::Utc::now(),
        };

        if let Err(err) = service.store_conversation_key(key).await {
            return HttpResponse::BadRequest().json(err);
        }
    }

    HttpResponse::Created().json(created)
}

#[get("")]
async fn get_conversations(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    let auth_user = user.into_inner();

    let school_id = match req.extensions().get::<ObjectId>().copied() {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(AppError { 
            message: "School ID not found".to_string() 
        }),
    };

    let auth_user_id = match ObjectId::parse_str(&auth_user.id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(AppError { 
            message: "Invalid user ID".to_string() 
        }),
    };

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let skip = (page - 1) * limit;

    let db = get_database(&req, &state);
    let service = ConversationService::new(&db);

    let extra_match = doc! {
        "school_id": school_id,
        "participants.user.id": auth_user_id
    };

    let result = match service.get_all(None, Some(limit), Some(skip), Some(extra_match)).await {
        Ok(data) => data,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    HttpResponse::Ok().json(result)
}

#[get("/{id}")]
async fn get_conversation(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let auth_user = user.into_inner();

    let id = path.into_inner();
    let db = get_database(&req, &state);
    let service = ConversationService::new(&db);

    let auth_user_id = match ObjectId::parse_str(&auth_user.id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(AppError { 
            message: "Invalid user ID".to_string() 
        }),
    };

    // Fetch conversation with participant check in one query
    let extra_match = doc! {
        "participants.user.id": auth_user_id
    };

    let conversation = match service.find_one(Some(&IdType::String(id)), Some(extra_match)).await {
        Ok(conv) => conv,
        Err(err) => {
            if err.message.contains("not found") {
                return HttpResponse::Forbidden().json(AppError { 
                    message: "You are not a participant in this conversation".to_string() 
                });
            }
            return HttpResponse::NotFound().json(err);
        }
    };

    HttpResponse::Ok().json(conversation)
}

#[get("/{id}/key")]
async fn get_conversation_key(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let auth_user = user.into_inner();

    let conversation_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(AppError { 
            message: "Invalid conversation ID".to_string() 
        }),
    };

    let auth_user_id = match ObjectId::parse_str(&auth_user.id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(AppError { 
            message: "Invalid user ID".to_string() 
        }),
    };

    let db = get_database(&req, &state);
    let service = ConversationService::new(&db);

    // Get key directly - if it doesn't exist, user is not a participant
    let key = match service.get_conversation_key(conversation_id, auth_user_id).await {
        Ok(k) => k,
        Err(err) => {
            if err.message.contains("not found") {
                return HttpResponse::Forbidden().json(AppError { 
                    message: "You are not a participant in this conversation".to_string() 
                });
            }
            return HttpResponse::NotFound().json(err);
        }
    };

    HttpResponse::Ok().json(key)
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(SchoolTokenMiddleware)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_conversation)
            .service(get_conversations)
            .service(get_conversation)
            .service(get_conversation_key),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/conversations").configure(blueprint));
}
