use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    config::state::AppState,
    domain::{auth_user::AuthUserDto, user_public_key::PublicKeyInfo},
    errors::AppError,
    middleware::school_token_middleware::OptionalSchoolTokenMiddleware,
    services::user_public_key_service::UserPublicKeyService,
};

#[derive(Debug, Deserialize)]
struct UploadPublicKeyRequest {
    public_key: String,
    key_algorithm: String,
}

#[derive(Debug, Serialize)]
struct UploadPublicKeyResponse {
    message: String,
    user_id: String,
}

#[derive(Debug, Serialize)]
struct GetPublicKeysResponse {
    public_keys: Vec<PublicKeyInfo>,
}

#[derive(Debug, Serialize)]
struct PublicKeyNotFoundError {
    message: String,
    missing_user_ids: Vec<String>,
}

#[post("/public-key")]
async fn upload_public_key(
    _req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    body: web::Json<UploadPublicKeyRequest>,
) -> impl Responder {
    let auth_user = user.into_inner();

    let user_id = match ObjectId::parse_str(&auth_user.id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(AppError {
                message: "Invalid user ID".to_string(),
            })
        }
    };

    // Use main database for public keys (not school-specific)
    let db = &state.db.main_db();
    let service = UserPublicKeyService::new(db);

    match service
        .upsert_public_key(user_id, body.public_key.clone(), body.key_algorithm.clone())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(UploadPublicKeyResponse {
            message: "Public key uploaded successfully".to_string(),
            user_id: auth_user.id,
        }),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/public-keys")]
async fn get_public_keys(
    _req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user_ids_str = match query.get("user_ids") {
        Some(ids) => ids,
        None => {
            return HttpResponse::BadRequest().json(AppError {
                message: "user_ids parameter is required".to_string(),
            })
        }
    };

    // Parse comma-separated user IDs
    let user_ids: Vec<ObjectId> = user_ids_str
        .split(',')
        .filter_map(|id| ObjectId::parse_str(id.trim()).ok())
        .collect();

    if user_ids.is_empty() {
        return HttpResponse::BadRequest().json(AppError {
            message: "At least one valid user ID is required".to_string(),
        });
    }

    if user_ids.len() > 50 {
        return HttpResponse::BadRequest().json(AppError {
            message: "Maximum 50 user IDs allowed per request".to_string(),
        });
    }

    // Use main database for public keys (not school-specific)
    let db = &state.db.main_db();
    let service = UserPublicKeyService::new(db);

    match service.get_public_keys(user_ids.clone()).await {
        Ok(public_keys) => HttpResponse::Ok().json(GetPublicKeysResponse { public_keys }),
        Err(err) => {
            // Extract missing user IDs from error message
            if err.message.contains("Public key not found for users:") {
                let missing_part = err.message.replace("Public key not found for users: ", "");
                let missing_user_ids: Vec<String> =
                    missing_part.split(", ").map(|s| s.to_string()).collect();

                return HttpResponse::NotFound().json(PublicKeyNotFoundError {
                    message: err.message,
                    missing_user_ids,
                });
            }

            HttpResponse::BadRequest().json(err)
        }
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .wrap(OptionalSchoolTokenMiddleware)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(upload_public_key)
            .service(get_public_keys),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("").configure(blueprint));
}
