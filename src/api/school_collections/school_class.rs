use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::class::{Class, UpdateClass},
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::class_repo::ClassRepo,
    services::{class_service::ClassService, event_service::EventService},
};

#[get("")]
async fn get_all_school_classes(
    req: actix_web::HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Get school claims from extensions (set by SchoolTokenMiddleware)
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    // Use school database from claims
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .get_all_classes(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_school_classes(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_active_classes().await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_school_class_by_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_by_id(&class_id).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_school_class_by_username(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let username = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_by_username(&username).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_school_class_by_code(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let code = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_by_code(&code).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_school_class(
    req: actix_web::HttpRequest,
    data: web::Json<Class>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Set the school_id from the token to ensure consistency
    let mut class_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    class_data.school_id = Some(school_id);

    match service.create_class(class_data).await {
        Ok(class) => {
            // Broadcast created class event
            let class_clone = class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(class)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_school_class(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.update_class(&class_id, data.into_inner()).await {
        Ok(class) => {
            // Broadcast updated class event
            let class_clone = class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(class)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_school_class(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Get class before deletion for broadcasting
    let class_before_delete = repo.find_by_id(&class_id).await.ok().flatten();

    match service.delete_class(&class_id).await {
        Ok(_) => {
            // Broadcast deleted class event
            if let Some(class) = class_before_delete {
                let state_clone = state.clone();

                actix_rt::spawn(async move {
                    if let Some(id) = class.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "class",
                            &id.to_hex(),
                            &class,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Class deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count")]
async fn count_school_classes(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .count_classes_by_school_id(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/classes")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // Public routes (read-only)
            .service(get_all_school_classes) // GET /school/classes - Get all classes in school
            .service(get_active_school_classes) // GET /school/classes/active - Get active classes in school
            .service(get_school_class_by_id) // GET /school/classes/{id} - Get class by ID in school
            .service(get_school_class_by_username) // GET /school/classes/username/{username} - Get class by username in school
            .service(get_school_class_by_code) // GET /school/classes/code/{code} - Get class by code in school
            .service(count_school_classes) // GET /school/classes/stats/count - Count classes in school
            // Protected routes (require school token)
            .service(create_school_class) // POST /school/classes - Create new class in school
            .service(update_school_class) // PUT /school/classes/{id} - Update class in school
            .service(delete_school_class), // DELETE /school/classes/{id} - Delete class in school
    );
}
