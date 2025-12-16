use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        subjects::subject_progress_tracking_config::{
            DefaultSubjectProgressThresholds, SubjectProgressTrackingConfig,
            UpdateSubjectProgressTrackingConfig,
        },
    },
    models::{
        api_request_model::ReferenceIdsRequest, id_model::IdType, request_error_model::ReqErrModel,
    },
    repositories::subjects::subject_progress_configs_repo::SubjectProgressConfigsRepo,
    services::event_service::EventService,
    services::subjects::subject_progress_configs_service::SubjectProgressConfigsService,
};

#[get("")]
async fn get_all_configs(state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    match service.get_all_configs().await {
        Ok(configs) => HttpResponse::Ok().json(configs),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_config_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    let config_id = IdType::from_string(path.into_inner());

    match service.get_config_by_id(&config_id).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}")]
async fn get_config_by_subject_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_config_by_id(&subject_id).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_config(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SubjectProgressTrackingConfig>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    match service.create_config(data.into_inner()).await {
        Ok(config) => {
            // 🔔 Broadcast real-time event
            let config_clone = config.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = config_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "subject_progress_config",
                        &id.to_hex(),
                        &config_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(config)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/default")]
async fn create_default_config(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<DefaultSubjectProgressThresholds>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    match service
        .get_or_create_default_config(data.into_inner())
        .await
    {
        Ok(config) => {
            // 🔔 Broadcast real-time event
            let config_clone = config.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = config_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "subject_progress_config",
                        &id.to_hex(),
                        &config_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(config)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_config(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSubjectProgressTrackingConfig>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let config_id = IdType::from_string(path.into_inner());
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    match service.update_config(&config_id, data.into_inner()).await {
        Ok(config) => {
            // 🔔 Broadcast real-time event
            let config_clone = config.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = config_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject_progress_config",
                        &id.to_hex(),
                        &config_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(config)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_config(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let config_id = IdType::from_string(path.into_inner());
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    // Get config before deletion for broadcasting
    let config_before_delete = repo.find_by_id(&config_id).await.ok().flatten();

    match service.delete_config(&config_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(config) = config_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = config.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "subject_progress_config",
                            &id.to_hex(),
                            &config,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Progress config deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/by-reference-ids")]
async fn get_configs_by_reference_ids(
    data: web::Json<ReferenceIdsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    // Parse into ObjectIds
    let mut object_ids = Vec::new();
    for id_str in &data.reference_ids {
        match ObjectId::parse_str(id_str) {
            Ok(oid) => object_ids.push(oid),
            Err(_) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: format!("Invalid ObjectId: {}", id_str),
                });
            }
        }
    }

    match service.get_configs_by_reference_ids(&object_ids).await {
        Ok(configs) => HttpResponse::Ok().json(configs),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/reference/{id}")]
async fn get_config_by_reference_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectProgressConfigsRepo::new(&state.db.main_db());
    let service = SubjectProgressConfigsService::new(&repo);

    let reference_id = IdType::from_string(path.into_inner());

    match service.get_config_by_reference_id(&reference_id).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subject-progress-configs")
            // Public routes
            .service(get_configs_by_reference_ids) // POST /subject-progress-configs/by-reference-ids
            .service(get_config_by_reference_id) // GET /subject-progress-configs/reference/{id}
            .service(get_all_configs) // GET /subject-progress-configs
            .service(get_config_by_subject_id) // GET /subject-progress-configs/subject/{subject_id}
            .service(get_config_by_id) // GET /subject-progress-configs/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_config) // POST /subject-progress-configs
            .service(create_default_config) // POST /subject-progress-configs/default
            .service(update_config) // PUT /subject-progress-configs/{id}
            .service(delete_config), // DELETE /subject-progress-configs/{id}
    );
}
