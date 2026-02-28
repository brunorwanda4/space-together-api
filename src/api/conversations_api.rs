use std::str::FromStr;

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        common_details::UserRole,
        conversation::{Conversation, ConversationKey},
    },
    errors::AppError,
    middleware::school_token_middleware::OptionalSchoolTokenMiddleware,
    models::{id_model::IdType, school_token_model::SchoolToken},
    schema::common_schema::ActorRef,
    services::conversation_service::ConversationService,
    utils::db_utils::get_database,
};

#[derive(Debug, Deserialize)]
struct CreateConversationRequest {
    participants: Vec<ActorRef>,
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
    let school_id = req.extensions()
        .get::<SchoolToken>()
        .and_then(|token| ObjectId::from_str(&token.id).ok());

    // Validate minimum participants
    if body.participants.len() < 2 {
        return HttpResponse::BadRequest().json(AppError { 
            message: "Minimum 2 participants required".to_string() 
        });
    }

    // Validate maximum participants for group conversations
    if body.participants.len() > 50 {
        return HttpResponse::BadRequest().json(AppError { 
            message: "Maximum 50 participants allowed for group conversations".to_string() 
        });
    }

    // Validate group conversation name
    if body.is_group {
        if body.name.is_none() || body.name.as_ref().unwrap().trim().is_empty() {
            return HttpResponse::BadRequest().json(AppError { 
                message: "Group name is required for group conversations".to_string() 
            });
        }
        if let Some(ref name) = body.name {
            if name.len() > 100 {
                return HttpResponse::BadRequest().json(AppError { 
                    message: "Group name must be 1-100 characters".to_string() 
                });
            }
        }
    } else {
        // For non-group conversations, name must be null
        if body.name.is_some() {
            return HttpResponse::BadRequest().json(AppError { 
                message: "Name must be null for direct conversations".to_string() 
            });
        }
    }

    // Check for duplicate participants
    let mut unique_participants = std::collections::HashSet::new();
    for participant in &body.participants {
        let key = participant.id.to_hex();
        if !unique_participants.insert(key) {
            return HttpResponse::BadRequest().json(AppError { 
                message: "Duplicate participants are not allowed".to_string() 
            });
        }
    }

    // Validate authenticated user is in participants
    let user_in_participants = body.participants.iter().any(|p| {
        p.id.to_hex() == auth_user.id
    });

    if !user_in_participants {
        return HttpResponse::BadRequest().json(AppError { 
            message: "You must be a participant in the conversation".to_string() 
        });
    }

    // Validate encrypted keys match participants
    if body.encrypted_keys.len() != body.participants.len() {
        return HttpResponse::BadRequest().json(AppError { 
            message: "Must provide exactly one encrypted key per participant".to_string() 
        });
    }

    // Validate each encrypted key matches a participant
    for key_data in &body.encrypted_keys {
        let matching_participant = body.participants.iter().find(|p| {
            p.id.to_hex() == key_data.user_id && p.role == key_data.user_role
        });

        if matching_participant.is_none() {
            return HttpResponse::BadRequest().json(AppError { 
                message: format!(
                    "Encrypted key for user {} with role {:?} does not match any participant",
                    key_data.user_id, key_data.user_role
                )
            });
        }

        // Validate base64 format
        if base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &key_data.encrypted_key).is_err() {
            return HttpResponse::UnprocessableEntity().json(AppError { 
                message: "Invalid encrypted key format. Must be valid base64.".to_string() 
            });
        }
    }

    let db = get_database(&req, &state);
    let service = ConversationService::new(&db);

    // Check for duplicate 1-on-1 conversations
    if !body.is_group {
        let participant_ids: Vec<ObjectId> = body.participants.iter().map(|p| p.id).collect();
        
        let existing_filter = doc! {
            "is_group": false,
            "participants.id": { "$all": participant_ids.clone() },
            "$expr": { "$eq": [{ "$size": "$participants" }, 2] }
        };

        if let Ok(existing) = service.find_one(None, Some(existing_filter)).await {
            return HttpResponse::Conflict().json(serde_json::json!({
                "message": "Conversation already exists",
                "existing_conversation_id": existing.id.unwrap().to_hex()
            }));
        }
    }

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
                message: "Invalid user ID in encrypted_keys".to_string() 
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

    HttpResponse::Created().json(serde_json::json!({
        "conversation": created
    }))
}

#[get("")]
async fn get_conversations(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    let auth_user = user.into_inner();

    // School ID is optional - if present, fetch school-specific conversations
    // If not present, fetch main database conversations (cross-school/admin)
    let school_id = req.extensions()
        .get::<SchoolToken>()
        .and_then(|token| ObjectId::from_str(&token.id).ok());

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

    let mut extra_match = doc! {
        "participants.id": auth_user_id
    };

    // Filter by school_id if present, otherwise get main database conversations
    if let Some(school_id) = school_id {
        extra_match.insert("school_id", school_id);
    } else {
        // Fetch conversations without school_id (main database conversations)
        extra_match.insert("school_id", doc! { "$exists": false });
    }

    let result = match service.get_all_with_relations(Some(limit), Some(skip), Some(extra_match)).await {
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
        "participants.id": auth_user_id
    };

    let conversation = match service.find_one_with_relations(Some(&IdType::String(id)), Some(extra_match)).await {
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
            .wrap(OptionalSchoolTokenMiddleware) // Optional - allows both school and non-school contexts
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
