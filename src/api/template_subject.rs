use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        template_subject::{TemplateSubject, TemplateSubjectPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, template_subject_service::TemplateSubjectService},
    utils::api_utils::build_extra_match,
};

/// --------------------------------------
/// GET /template-subjects
/// --------------------------------------
#[get("")]
async fn get_all_template_subjects(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TemplateSubjectService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

#[get("/others")]
async fn get_all_template_subjects_with_others(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service
        .get_all_with_other(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// --------------------------------------
/// GET /template-subjects/{id}
/// --------------------------------------
#[get("/{id}")]
async fn get_template_subject_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.find_one_by_id(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// GET /template-subjects/{id}/others
/// --------------------------------------
#[get("/{id}/others")]
async fn get_template_subject_by_id_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.find_one_with_relations(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// GET /template-subjects/code/{code}
/// --------------------------------------
#[get("/code/{code}")]
async fn get_template_subject_by_code(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.find_one_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// GET /template-subjects/code/{code}/others
/// --------------------------------------
#[get("/code/{code}/others")]
async fn get_template_subject_by_code_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.find_one_with_relations_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// GET /template-subjects/prerequisite/{id}/others
/// --------------------------------------
#[get("/prerequisite/{id}/others")]
async fn find_many_by_prerequisite_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.find_many_by_prerequisite_with_relations(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// GET /template-subjects/prerequisite/{id}
/// --------------------------------------
#[get("/prerequisite/{id}")]
async fn find_many_by_prerequisite(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.find_many_by_prerequisite(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// POST /template-subjects
/// --------------------------------------
#[post("")]
async fn create_template_subject(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<TemplateSubject>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only ADMIN can create a template subject
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(subject) => {
            // 🔔 Broadcast creation event
            let subject_clone = subject.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "template_subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// --------------------------------------
/// PUT /template-subjects/{id}
/// --------------------------------------
#[put("/{id}")]
async fn update_template_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<TemplateSubjectPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only ADMIN can create a template subject
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    // Only ADMIN can update template subjects
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(&state.db.main_db());

    match service.update_subject(&id, &data.into_inner()).await {
        Ok(subject) => {
            // 🔔 Broadcast update event
            let subject_clone = subject.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "template_subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// --------------------------------------
/// DELETE /template-subjects/{id}
/// --------------------------------------
#[delete("/{id}")]
async fn delete_template_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only ADMIN can delete
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = TemplateSubjectService::new(&state.db.main_db());

    // Fetch before deletion for broadcast
    let before_delete = service.find_one_by_id(&id).await.ok();

    match service.delete_subject(&id).await {
        Ok(_) => {
            // 🔔 Broadcast deletion event
            if let Some(subject) = before_delete {
                let subject_clone = subject.clone();
                let state_clone = state.clone();

                actix_rt::spawn(async move {
                    if let Some(id) = subject_clone.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "template_subject",
                            &id.to_hex(),
                            &subject_clone,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Template subject deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template-subjects")
            .service(get_all_template_subjects) // GET /template-subjects - Get all template subjects
            .service(get_all_template_subjects_with_others) // GET /template-subjects/others - Get all template subjects with others
            .service(get_template_subject_by_code) // GET /template-subjects/code/{id}
            .service(get_template_subject_by_code_others) // GET /template-subjects/code/{id}/others
            .service(find_many_by_prerequisite_with_relations) // GET /template-subjects/prerequisite/{id}/others
            .service(find_many_by_prerequisite) // GET /template-subjects/prerequisite/{id}
            .service(get_template_subject_by_id) // GET /template-subjects/{id}
            .service(get_template_subject_by_id_others) // GET /template-subjects/{id}/others
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_template_subject) // POST /template-subjects
            .service(update_template_subject) // PUT /template-subjects/{id}
            .service(delete_template_subject), // DELETE /template-subjects/{id}
    );
}
