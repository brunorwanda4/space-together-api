use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        class_subject::{ClassSubject, ClassSubjectPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{class_subject_service::ClassSubjectService, event_service::EventService},
};

/// --------------------------------------
/// GET /class-subjects
/// --------------------------------------
#[get("")]
async fn get_all_class_subjects(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(&state.db.main_db());

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, None)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// --------------------------------------
/// GET /class-subjects/others
/// --------------------------------------
#[get("/others")]
async fn get_all_class_subjects_with_others(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassSubjectService::new(&state.db.main_db());

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// --------------------------------------
/// GET /class-subjects/{id}
/// --------------------------------------
#[get("/{id}")]
async fn get_class_subject_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = ClassSubjectService::new(&state.db.main_db());

    match service.find_one_by_id(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// --------------------------------------
/// GET /class-subjects/{id}/others
/// --------------------------------------
#[get("/{id}/others")]
async fn get_class_subject_by_id_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = ClassSubjectService::new(&state.db.main_db());

    match service.find_one_with_relations(&id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// --------------------------------------
/// GET /class-subjects/code/{code}
/// --------------------------------------
#[get("/code/{code}")]
async fn get_class_subject_by_code(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    let service = ClassSubjectService::new(&state.db.main_db());

    match service.find_one_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// --------------------------------------
/// GET /class-subjects/code/{code}/others
/// --------------------------------------
#[get("/code/{code}/others")]
async fn get_class_subject_by_code_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    let service = ClassSubjectService::new(&state.db.main_db());

    match service.find_one_with_relations_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// --------------------------------------
/// GET /class-subjects/teacher/{teacher_id}
/// --------------------------------------
#[get("/teacher/{teacher_id}")]
async fn get_by_teacher(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let teacher_id = IdType::from_string(path.into_inner());
    let service = ClassSubjectService::new(&state.db.main_db());

    match service.find_many_by_teacher(&teacher_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// --------------------------------------
/// POST /class-subjects
/// --------------------------------------
#[post("")]
async fn create_class_subject(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<ClassSubject>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let service = ClassSubjectService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(subject) => {
            // Broadcast event
            let clone = subject.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class_subject",
                        &id.to_hex(),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(subject)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// --------------------------------------
/// PUT /class-subjects/{id}
/// --------------------------------------
#[put("/{id}")]
async fn update_class_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ClassSubjectPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ClassSubjectService::new(&state.db.main_db());

    match service.update_subject(&id, &data.into_inner()).await {
        Ok(subject) => {
            // event
            let clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class_subject",
                        &id.to_hex(),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// --------------------------------------
/// DELETE /class-subjects/{id}
/// --------------------------------------
#[delete("/{id}")]
async fn delete_class_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ClassSubjectService::new(&state.db.main_db());

    let before = service.find_one_by_id(&id).await.ok();

    match service.delete_subject(&id).await {
        Ok(_) => {
            if let Some(subject) = before {
                let state_clone = state.clone();
                let clone = subject.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = clone.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "class_subject",
                            &id.to_hex(),
                            &clone,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Class subject deleted successfully"
            }))
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// --------------------------------------
/// Register Routes
/// --------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/class-subjects")
            .service(get_all_class_subjects)
            .service(get_all_class_subjects_with_others)
            .service(get_class_subject_by_code)
            .service(get_class_subject_by_code_others)
            .service(get_by_teacher)
            .service(get_class_subject_by_id)
            .service(get_class_subject_by_id_others)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_class_subject)
            .service(update_class_subject)
            .service(delete_class_subject),
    );
}
