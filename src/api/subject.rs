use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        subject::{
            BulkCheckIdentifiersRequest, BulkCheckIdentifiersResponse, BulkIdsRequest,
            BulkSubjectsForClassRequest, BulkSubjectsForMainSubjectRequest,
            BulkSubjectsForTeacherRequest, BulkSubjectsRequest, BulkTagsRequest,
            BulkUpdateActiveStatusRequest, BulkUpdateByClassRequest, BulkUpdateRequest, Subject,
            UpdateSubject,
        },
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
async fn get_all_subjects_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .get_all_subjects_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
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

/// Bulk delete multiple subjects
#[delete("/bulk")]
async fn delete_many_subjects(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkIdsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only admin or subject teachers can delete subjects
    for subject_id in &data.ids {
        if let Err(err) =
            crate::guards::role_guard::check_admin_or_subject_teacher(&logged_user, subject_id)
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to delete subject {}: {}", subject_id, err)
            }));
        }
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service.delete_many_subjects(ids).await {
        Ok(deleted_count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully deleted {} subjects", deleted_count)
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get subjects by multiple IDs
#[post("/bulk/get")]
async fn get_subjects_by_ids(
    data: web::Json<BulkIdsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service.get_subjects_by_ids(ids).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get subjects with relations by multiple IDs
#[post("/bulk/get/with-relations")]
async fn get_subjects_by_ids_with_relations(
    data: web::Json<BulkIdsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service.get_subjects_by_ids_with_relations(ids).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if identifiers (usernames/codes) already exist in bulk
#[post("/bulk/check-identifiers")]
async fn check_existing_identifiers(
    data: web::Json<BulkCheckIdentifiersRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .check_existing_identifiers(&data.usernames, &data.codes)
        .await
    {
        Ok((existing_usernames, existing_codes)) => {
            HttpResponse::Ok().json(BulkCheckIdentifiersResponse {
                existing_usernames,
                existing_codes,
            })
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update subjects by class ID
#[put("/bulk/class-update")]
async fn update_many_subjects_by_class_id(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateByClassRequest>,
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

    let class_id = IdType::from_string(data.class_id.clone());

    // Check if user has permission to update subjects for this class
    if let Err(err) = crate::guards::role_guard::check_class_access(&logged_user, &data.class_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": format!("No permission to update subjects for class: {}", err)
        }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .update_many_subjects_by_class_id(&class_id, data.update.clone())
        .await
    {
        Ok(updated_count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully updated {} subjects", updated_count)
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk activate/deactivate subjects
#[put("/bulk/active-status")]
async fn bulk_update_subjects_active_status(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateActiveStatusRequest>,
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

    // Check permissions for each subject
    for subject_id in &data.ids {
        if let Err(err) = crate::guards::role_guard::check_subject_access(&logged_user, subject_id)
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update subject {}: {}", subject_id, err)
            }));
        }
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service
        .bulk_update_subjects_active_status(ids, data.is_active)
        .await
    {
        Ok(updated_count) => {
            // Broadcast events for updated subjects
            let state_clone = state.clone();
            let subject_ids = data.ids.clone();
            actix_rt::spawn(async move {
                for subject_id in subject_ids {
                    // You might want to fetch and broadcast each subject, but for performance
                    // we'll just broadcast a generic update event
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject",
                        &subject_id,
                        &serde_json::json!({ "is_active": data.is_active }),
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Successfully updated {} subjects", updated_count)
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple subjects in bulk
#[post("/bulk")]
async fn create_many_subjects(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkSubjectsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service.create_many_subjects(data.subjects.clone()).await {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone(); // 👈 FIX
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple subjects with comprehensive validation
#[post("/bulk/validation")]
async fn create_many_subjects_with_validation(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkSubjectsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service
        .create_many_subjects_with_validation(data.subjects.clone())
        .await
    {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone(); // 👈 FIX
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple subjects for a specific class
#[post("/bulk/class")]
async fn create_many_subjects_for_class(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkSubjectsForClassRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let class_id = IdType::from_string(data.class_id.clone());

    match service
        .create_many_subjects_for_class(&class_id, data.subjects.clone())
        .await
    {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone(); // 👈 FIX
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple subjects for a specific teacher
#[post("/bulk/teacher")]
async fn create_many_subjects_for_teacher(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkSubjectsForTeacherRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let teacher_id = IdType::from_string(data.teacher_id.clone());

    match service
        .create_many_subjects_for_teacher(&teacher_id, data.subjects.clone())
        .await
    {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone(); // 👈 FIX
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple subjects for a specific main subject
#[post("/bulk/main-subject")]
async fn create_many_subjects_for_main_subject(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkSubjectsForMainSubjectRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let main_subject_id = IdType::from_string(data.main_subject_id.clone());

    match service
        .create_many_subjects_for_main_subject(&main_subject_id, data.subjects.clone())
        .await
    {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone(); // 👈 FIX
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update multiple subjects
#[put("/bulk")]
async fn update_many_subjects(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Unauthorized" }))
        }
    };

    let updates: Vec<(IdType, UpdateSubject)> = data
        .updates
        .iter()
        .map(|item| {
            let id = IdType::from_string(item.id.clone());
            (id, item.update.clone())
        })
        .collect();

    for (id, _) in &updates {
        if let Err(err) =
            crate::guards::role_guard::check_subject_access(&logged_user, &id.as_string())
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update subject: {}", err)
            }));
        }
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    match service.update_many_subjects(updates).await {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone(); // 👈 FIX
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk toggle subject status for multiple subjects
#[put("/bulk/toggle-status")]
async fn bulk_toggle_subjects_status(
    req: actix_web::HttpRequest,
    data: web::Json<BulkIdsRequest>,
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

    // Check permissions
    for subject_id in &data.ids {
        if let Err(err) = crate::guards::role_guard::check_subject_access(&logged_user, subject_id)
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update subject {}: {}", subject_id, err)
            }));
        }
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service.bulk_toggle_subjects_status(ids).await {
        Ok(subjects) => {
            // Clone subjects before spawning task
            let subjects_for_broadcast = subjects.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk add tags to multiple subjects
#[put("/bulk/add-tags")]
async fn bulk_add_subjects_tags(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTagsRequest>,
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

    for subject_id in &data.ids {
        if let Err(err) = crate::guards::role_guard::check_subject_access(&logged_user, subject_id)
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update subject {}: {}", subject_id, err)
            }));
        }
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service.bulk_add_subjects_tags(ids, data.tags.clone()).await {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(subjects)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk remove tags from multiple subjects
#[put("/bulk/remove-tags")]
async fn bulk_remove_subjects_tags(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTagsRequest>,
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

    for subject_id in &data.ids {
        if let Err(err) = crate::guards::role_guard::check_subject_access(&logged_user, subject_id)
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update subject {}: {}", subject_id, err)
            }));
        }
    }

    let repo = SubjectRepo::new(&state.db.main_db());
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service
        .bulk_remove_subjects_tags(ids, data.tags.clone())
        .await
    {
        Ok(subjects) => {
            let subjects_for_broadcast = subjects.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_broadcast {
                    if let Some(id) = subject.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "subject",
                            &id.to_hex(),
                            subject,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(subjects)
        }
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
            .service(check_existing_identifiers) // POST /subjects/bulk/check-identifiers - Check identifiers (public)
            .service(get_subjects_by_ids) // POST /subjects/bulk/get - Get subjects by multiple IDs (public)
            .service(get_subjects_by_ids_with_relations) // POST /subjects/bulk/get/with-relations - Get subjects with relations by multiple IDs (public)
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_subject) // POST /subjects - Create new subject (Admin/Staff/Teacher only)
            .service(update_subject) // PUT /subjects/{id} - Update subject (Admin/SubjectTeacher only)
            .service(update_subject_merged) // PUT /subjects/{id}/merged - Update subject with merge (Admin/SubjectTeacher only)
            .service(delete_subject) // DELETE /subjects/{id} - Delete subject (Admin/SubjectTeacher only)
            .service(toggle_subject_status) // PUT /subjects/{id}/toggle-status - Toggle subject active status
            .service(add_subject_tags) // PUT /subjects/{id}/add-tags - Add tags to subject
            .service(remove_subject_tags) // PUT /subjects/{id}/remove-tags - Remove tags from subject
            // Bulk operations (protected)
            .service(create_many_subjects) // POST /subjects/bulk - Create multiple subjects
            .service(create_many_subjects_with_validation) // POST /subjects/bulk/validation - Create multiple subjects with validation
            .service(create_many_subjects_for_class) // POST /subjects/bulk/class - Create multiple subjects for class
            .service(create_many_subjects_for_teacher) // POST /subjects/bulk/teacher - Create multiple subjects for teacher
            .service(create_many_subjects_for_main_subject) // POST /subjects/bulk/main-subject - Create multiple subjects for main subject
            .service(update_many_subjects) // PUT /subjects/bulk - Update multiple subjects
            .service(delete_many_subjects) // DELETE /subjects/bulk - Delete multiple subjects
            .service(update_many_subjects_by_class_id) // PUT /subjects/bulk/class-update - Update subjects by class ID
            .service(bulk_update_subjects_active_status) // PUT /subjects/bulk/active-status - Bulk activate/deactivate subjects
            .service(bulk_toggle_subjects_status) // PUT /subjects/bulk/toggle-status - Bulk toggle subject status
            .service(bulk_add_subjects_tags) // PUT /subjects/bulk/add-tags - Bulk add tags to subjects
            .service(bulk_remove_subjects_tags), // PUT /subjects/bulk/remove-tags - Bulk remove tags from subjects
    );
}
