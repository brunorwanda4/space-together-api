use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        subject::{Subject, UpdateSubject},
    },
    models::{api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel},
    repositories::subject_repo::SubjectRepo,
    services::{event_service::EventService, subject_service::SubjectService},
};

#[get("")]
async fn get_all_subjects(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .get_all_subjects(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-relations")]
async fn get_all_subjects_with_relations(state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service.get_all_subjects_with_relations().await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_subjects(state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service.get_active_subjects().await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_subject_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_subject_by_id(&subject_id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-relations")]
async fn get_subject_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_subject_by_id_with_relations(&subject_id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_subject_by_username(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let username = path.into_inner();

    match service.get_subject_by_username(&username).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}/with-relations")]
async fn get_subject_by_username_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let username = path.into_inner();

    match service
        .get_subject_by_username_with_relations(&username)
        .await
    {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_subject_by_code(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let code = path.into_inner();

    match service.get_subject_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}/with-relations")]
async fn get_subject_by_code_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let code = path.into_inner();

    match service.get_subject_by_code_with_relations(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/class/{class_id}")]
async fn get_subjects_by_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.get_subjects_by_class_id(&class_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_subjects_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.get_subjects_by_creator_id(&creator_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/teacher/{teacher_id}")]
async fn get_subjects_by_teacher_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let teacher_id = IdType::from_string(path.into_inner());

    match service.get_subjects_by_class_teacher_id(&teacher_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/main-subject/{main_subject_id}")]
async fn get_subjects_by_main_subject_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let main_subject_id = IdType::from_string(path.into_inner());

    match service
        .get_subjects_by_main_subject_id(&main_subject_id)
        .await
    {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/type/{subject_type}")]
async fn get_subjects_by_subject_type(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let subject_type_str = path.into_inner();

    // Convert string to SubjectCategory
    let subject_type = match subject_type_str.to_lowercase().as_str() {
        "science" => crate::domain::subjects::subject_category::SubjectCategory::Science,
        "technology" => crate::domain::subjects::subject_category::SubjectCategory::Technology,
        "engineering" => crate::domain::subjects::subject_category::SubjectCategory::Engineering,
        "mathematics" => crate::domain::subjects::subject_category::SubjectCategory::Mathematics,
        "language" => crate::domain::subjects::subject_category::SubjectCategory::Language,
        "socialscience" => {
            crate::domain::subjects::subject_category::SubjectCategory::SocialScience
        }
        "arts" => crate::domain::subjects::subject_category::SubjectCategory::Arts,
        "tvet" => crate::domain::subjects::subject_category::SubjectCategory::TVET,
        custom => {
            crate::domain::subjects::subject_category::SubjectCategory::Other(custom.to_string())
        }
    };

    match service.get_subjects_by_subject_type(&subject_type).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_subject(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Subject>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // ✅ Only admin, staff, or teachers can create subjects
    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service.create_subject(data.into_inner()).await {
        Ok(subject) => {
            // ✅ Broadcast created subject event
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_subject(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSubject>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_subject_id_str = path.into_inner();

    // Check if user has permission to update subject
    if let Err(err) =
        crate::guards::role_guard::check_subject_access(&logged_user, &target_subject_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_subject_id = IdType::from_string(target_subject_id_str);
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .update_subject(&target_subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // 🔔 Broadcast real-time event
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/merged")]
async fn update_subject_merged(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSubject>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_subject_id_str = path.into_inner();

    // Check if user has permission to update subject
    if let Err(err) =
        crate::guards::role_guard::check_subject_access(&logged_user, &target_subject_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_subject_id = IdType::from_string(target_subject_id_str);
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .update_subject_merged(&target_subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // 🔔 Broadcast real-time event
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let target_subject_id_str = path.into_inner();

    // Only admin or subject teacher can delete subjects
    if let Err(err) = crate::guards::role_guard::check_admin_or_subject_teacher(
        &logged_user,
        &target_subject_id_str,
    ) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_subject_id = IdType::from_string(target_subject_id_str);
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    // Get subject before deletion for broadcasting
    let subject_before_delete = repo.find_by_id(&target_subject_id).await.ok().flatten();

    match service.delete_subject(&target_subject_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(subject) = subject_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = subject.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            &subject,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Subject deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/toggle-status")]
async fn toggle_subject_status(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_subject_id_str = path.into_inner();

    // Check if user has permission to update subject
    if let Err(err) =
        crate::guards::role_guard::check_subject_access(&logged_user, &target_subject_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_subject_id = IdType::from_string(target_subject_id_str);
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service.toggle_subject_status(&target_subject_id).await {
        Ok(subject) => {
            // 🔔 Broadcast real-time event
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/add-tags")]
async fn add_subject_tags(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_subject_id_str = path.into_inner();

    // Check if user has permission to update subject
    if let Err(err) =
        crate::guards::role_guard::check_subject_access(&logged_user, &target_subject_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_subject_id = IdType::from_string(target_subject_id_str);
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .add_subject_tags(&target_subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // 🔔 Broadcast real-time event
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/remove-tags")]
async fn remove_subject_tags(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_subject_id_str = path.into_inner();

    // Check if user has permission to update subject
    if let Err(err) =
        crate::guards::role_guard::check_subject_access(&logged_user, &target_subject_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_subject_id = IdType::from_string(target_subject_id_str);
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .remove_subject_tags(&target_subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // 🔔 Broadcast real-time event
            let subject_clone = subject.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = subject_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject",
                        &id.to_hex(),
                        &subject_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subject)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-class/{class_id}")]
async fn count_subjects_by_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.count_subjects_by_class_id(&class_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_subjects_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.count_subjects_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-teacher/{teacher_id}")]
async fn count_subjects_by_teacher_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let teacher_id = IdType::from_string(path.into_inner());

    match service
        .count_subjects_by_class_teacher_id(&teacher_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-main-subject/{main_subject_id}")]
async fn count_subjects_by_main_subject_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let main_subject_id = IdType::from_string(path.into_inner());

    match service
        .count_subjects_by_main_subject_id(&main_subject_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subjects")
            // Public routes (read-only)
            .service(get_all_subjects) // GET /subjects - Get all subjects
            .service(get_all_subjects_with_relations) // GET /subjects/with-relations - Get all subjects with relations
            .service(get_active_subjects) // GET /subjects/active - Get all active subjects
            .service(get_subject_by_id) // GET /subjects/{id} - Get subject by ID
            .service(get_subject_by_id_with_relations) // GET /subjects/{id}/with-relations - Get subject by ID with relations
            .service(get_subject_by_username) // GET /subjects/username/{username} - Get subject by username
            .service(get_subject_by_username_with_relations) // GET /subjects/username/{username}/with-relations - Get subject by username with relations
            .service(get_subject_by_code) // GET /subjects/code/{code} - Get subject by code
            .service(get_subject_by_code_with_relations) // GET /subjects/code/{code}/with-relations - Get subject by code with relations
            .service(get_subjects_by_class_id) // GET /subjects/class/{class_id} - Get subjects by class ID
            .service(get_subjects_by_creator_id) // GET /subjects/creator/{creator_id} - Get subjects by creator ID
            .service(get_subjects_by_teacher_id) // GET /subjects/teacher/{teacher_id} - Get subjects by teacher ID
            .service(get_subjects_by_main_subject_id) // GET /subjects/main-subject/{main_subject_id} - Get subjects by main subject ID
            .service(get_subjects_by_subject_type) // GET /subjects/type/{subject_type} - Get subjects by subject type
            .service(count_subjects_by_class_id) // GET /subjects/stats/count-by-class/{class_id} - Count subjects by class ID
            .service(count_subjects_by_creator_id) // GET /subjects/stats/count-by-creator/{creator_id} - Count subjects by creator ID
            .service(count_subjects_by_teacher_id) // GET /subjects/stats/count-by-teacher/{teacher_id} - Count subjects by teacher ID
            .service(count_subjects_by_main_subject_id) // GET /subjects/stats/count-by-main-subject/{main_subject_id} - Count subjects by main subject ID
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_subject) // POST /subjects - Create new subject (Admin/Staff/Teacher only)
            .service(update_subject) // PUT /subjects/{id} - Update subject (Admin/SubjectTeacher only)
            .service(update_subject_merged) // PUT /subjects/{id}/merged - Update subject with merge (Admin/SubjectTeacher only)
            .service(delete_subject) // DELETE /subjects/{id} - Delete subject (Admin/SubjectTeacher only)
            .service(toggle_subject_status) // PUT /subjects/{id}/toggle-status - Toggle subject active status
            .service(add_subject_tags) // PUT /subjects/{id}/add-tags - Add tags to subject
            .service(remove_subject_tags), // PUT /subjects/{id}/remove-tags - Remove tags from subject
    );
}
