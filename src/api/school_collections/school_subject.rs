use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::subject::{
        BulkCheckIdentifiersRequest, BulkCheckIdentifiersResponse, BulkIdsRequest,
        BulkSubjectsForClassRequest, BulkSubjectsRequest, BulkUpdateRequest, Subject,
        UpdateSubject,
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::subject_repo::SubjectRepo,
    services::{event_service::EventService, subject_service::SubjectService},
};

#[get("")]
async fn get_all_school_subjects(
    req: actix_web::HttpRequest,
    query: web::Query<RequestQuery>,
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
    let repo = SubjectRepo::new(&school_db);
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
async fn get_all_school_subjects_with_relations(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_all_subjects_with_relations().await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_school_subjects(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_active_subjects().await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_school_subject_by_id(
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subject_by_id(&subject_id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-relations")]
async fn get_school_subject_by_id_with_relations(
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subject_by_id_with_relations(&subject_id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_school_subject_by_username(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subject_by_username(&username).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}/with-relations")]
async fn get_school_subject_by_username_with_relations(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .get_subject_by_username_with_relations(&username)
        .await
    {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_school_subject_by_code(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subject_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}/with-relations")]
async fn get_school_subject_by_code_with_relations(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subject_by_code_with_relations(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/class/{class_id}")]
async fn get_school_subjects_by_class_id(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subjects_by_class_id(&class_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_school_subjects_by_creator_id(
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

    let creator_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subjects_by_creator_id(&creator_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/teacher/{teacher_id}")]
async fn get_school_subjects_by_teacher_id(
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

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.get_subjects_by_class_teacher_id(&teacher_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/main-subject/{main_subject_id}")]
async fn get_school_subjects_by_main_subject_id(
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

    let main_subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .get_subjects_by_main_subject_id(&main_subject_id)
        .await
    {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/type/{subject_type}")]
async fn get_school_subjects_by_subject_type(
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

    let subject_type_str = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

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
async fn create_school_subject(
    req: actix_web::HttpRequest,
    data: web::Json<Subject>,
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    // Note: Subjects don't have school_id field, they are linked through classes
    let subject_data = data.into_inner();

    match service.create_subject(subject_data).await {
        Ok(subject) => {
            // Broadcast created subject event
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
async fn update_school_subject(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSubject>,
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.update_subject(&subject_id, data.into_inner()).await {
        Ok(subject) => {
            // Broadcast updated subject event
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
async fn update_school_subject_merged(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSubject>,
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .update_subject_merged(&subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // Broadcast updated subject event
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
async fn delete_school_subject(
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    // Get subject before deletion for broadcasting
    let subject_before_delete = repo.find_by_id(&subject_id).await.ok().flatten();

    match service.delete_subject(&subject_id).await {
        Ok(_) => {
            // Broadcast deleted subject event
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
async fn toggle_school_subject_status(
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.toggle_subject_status(&subject_id).await {
        Ok(subject) => {
            // Broadcast updated subject event
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
async fn add_school_subject_tags(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .add_subject_tags(&subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // Broadcast updated subject event
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
async fn remove_school_subject_tags(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
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

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .remove_subject_tags(&subject_id, data.into_inner())
        .await
    {
        Ok(subject) => {
            // Broadcast updated subject event
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

#[get("/stats/count")]
async fn count_school_subjects(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    // Since subjects don't have school_id, we count all subjects in the school database
    match service.get_all_subjects(None, None, None).await {
        Ok(subjects) => HttpResponse::Ok().json(serde_json::json!({ "count": subjects.len() })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-class/{class_id}")]
async fn count_school_subjects_by_class_id(
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.count_subjects_by_class_id(&class_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_school_subjects_by_creator_id(
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

    let creator_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service.count_subjects_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-teacher/{teacher_id}")]
async fn count_school_subjects_by_teacher_id(
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

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .count_subjects_by_class_teacher_id(&teacher_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-main-subject/{main_subject_id}")]
async fn count_school_subjects_by_main_subject_id(
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

    let main_subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    match service
        .count_subjects_by_main_subject_id(&main_subject_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple subjects for school
#[post("/bulk")]
async fn create_many_school_subjects(
    req: actix_web::HttpRequest,
    data: web::Json<BulkSubjectsRequest>,
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    // Note: Subjects don't have school_id field, they are linked through classes
    match service.create_many_subjects(data.subjects.clone()).await {
        Ok(subjects) => {
            let state_clone = state.clone();
            let subjects_for_spawn = subjects.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_spawn {
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

/// Create multiple subjects with validation for school
#[post("/bulk/validation")]
async fn create_many_school_subjects_with_validation(
    req: actix_web::HttpRequest,
    data: web::Json<BulkSubjectsRequest>,
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    // Note: Subjects don't have school_id field, they are linked through classes
    match service
        .create_many_subjects_with_validation(data.subjects.clone())
        .await
    {
        Ok(subjects) => {
            let state_clone = state.clone();
            let subjects_for_spawn = subjects.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_spawn {
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

/// Create multiple subjects for a specific class in school
#[post("/bulk/class")]
async fn create_many_school_subjects_for_class(
    req: actix_web::HttpRequest,
    data: web::Json<BulkSubjectsForClassRequest>,
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    let class_id = IdType::from_string(data.class_id.clone());

    // Note: Subjects don't have school_id field, they are linked through classes
    match service
        .create_many_subjects_for_class(&class_id, data.subjects.clone())
        .await
    {
        Ok(subjects) => {
            let state_clone = state.clone();
            let subjects_for_spawn = subjects.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_spawn {
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

/// Bulk update multiple subjects for school
#[put("/bulk")]
async fn update_many_school_subjects(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateRequest>,
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    let updates: Vec<(IdType, UpdateSubject)> = data
        .updates
        .iter()
        .map(|item| {
            let id = IdType::from_string(item.id.clone());
            (id, item.update.clone())
        })
        .collect();

    match service.update_many_subjects(updates).await {
        Ok(subjects) => {
            let state_clone = state.clone();
            let subjects_for_spawn = subjects.clone();

            actix_rt::spawn(async move {
                for subject in &subjects_for_spawn {
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

/// Bulk toggle subject status for multiple subjects in school
#[put("/bulk/toggle-status")]
async fn bulk_toggle_school_subjects_status(
    req: actix_web::HttpRequest,
    data: web::Json<BulkIdsRequest>,
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
    let repo = SubjectRepo::new(&school_db);
    let service = SubjectService::new(&repo);

    let ids: Vec<IdType> = data
        .ids
        .iter()
        .map(|id| IdType::from_string(id.clone()))
        .collect();

    match service.bulk_toggle_subjects_status(ids).await {
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

/// Get subjects by multiple IDs in school
#[post("/bulk/get")]
async fn get_school_subjects_by_ids(
    req: actix_web::HttpRequest,
    data: web::Json<BulkIdsRequest>,
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
    let repo = SubjectRepo::new(&school_db);
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

/// Check if identifiers (usernames/codes) already exist in bulk for school
#[post("/bulk/check-identifiers")]
async fn check_school_existing_identifiers(
    req: actix_web::HttpRequest,
    data: web::Json<BulkCheckIdentifiersRequest>,
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
    let repo = SubjectRepo::new(&school_db);
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

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/subjects")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // Public routes (read-only)
            .service(get_all_school_subjects) // GET /school/subjects - Get all subjects in school
            .service(get_all_school_subjects_with_relations) // GET /school/subjects/with-relations - Get all subjects with relations in school
            .service(get_active_school_subjects) // GET /school/subjects/active - Get active subjects in school
            .service(get_school_subject_by_id) // GET /school/subjects/{id} - Get subject by ID in school
            .service(get_school_subject_by_id_with_relations) // GET /school/subjects/{id}/with-relations - Get subject by ID with relations in school
            .service(get_school_subject_by_username) // GET /school/subjects/username/{username} - Get subject by username in school
            .service(get_school_subject_by_username_with_relations) // GET /school/subjects/username/{username}/with-relations - Get subject by username with relations in school
            .service(get_school_subject_by_code) // GET /school/subjects/code/{code} - Get subject by code in school
            .service(get_school_subject_by_code_with_relations) // GET /school/subjects/code/{code}/with-relations - Get subject by code with relations in school
            .service(get_school_subjects_by_class_id) // GET /school/subjects/class/{class_id} - Get subjects by class ID in school
            .service(get_school_subjects_by_creator_id) // GET /school/subjects/creator/{creator_id} - Get subjects by creator ID in school
            .service(get_school_subjects_by_teacher_id) // GET /school/subjects/teacher/{teacher_id} - Get subjects by teacher ID in school
            .service(get_school_subjects_by_main_subject_id) // GET /school/subjects/main-subject/{main_subject_id} - Get subjects by main subject ID in school
            .service(get_school_subjects_by_subject_type) // GET /school/subjects/type/{subject_type} - Get subjects by subject type in school
            .service(count_school_subjects) // GET /school/subjects/stats/count - Count subjects in school
            .service(count_school_subjects_by_class_id) // GET /school/subjects/stats/count-by-class/{class_id} - Count subjects by class ID in school
            .service(count_school_subjects_by_creator_id) // GET /school/subjects/stats/count-by-creator/{creator_id} - Count subjects by creator ID in school
            .service(count_school_subjects_by_teacher_id) // GET /school/subjects/stats/count-by-teacher/{teacher_id} - Count subjects by teacher ID in school
            .service(count_school_subjects_by_main_subject_id) // GET /school/subjects/stats/count-by-main-subject/{main_subject_id} - Count subjects by main subject ID in school
            // Protected routes (require school token)
            .service(create_school_subject) // POST /school/subjects - Create new subject in school
            .service(update_school_subject) // PUT /school/subjects/{id} - Update subject in school
            .service(update_school_subject_merged) // PUT /school/subjects/{id}/merged - Update subject with merge in school
            .service(delete_school_subject) // DELETE /school/subjects/{id} - Delete subject in school
            .service(toggle_school_subject_status) // PUT /school/subjects/{id}/toggle-status - Toggle subject status in school
            .service(add_school_subject_tags) // PUT /school/subjects/{id}/add-tags - Add tags to subject in school
            .service(remove_school_subject_tags) // PUT /school/subjects/{id}/remove-tags - Remove tags from subject in school
            // Bulk operations for school
            .service(create_many_school_subjects) // POST /school/subjects/bulk - Create multiple subjects in school
            .service(create_many_school_subjects_with_validation) // POST /school/subjects/bulk/validation - Create multiple subjects with validation in school
            .service(create_many_school_subjects_for_class) // POST /school/subjects/bulk/class - Create multiple subjects for class in school
            .service(update_many_school_subjects) // PUT /school/subjects/bulk - Update multiple subjects in school
            .service(bulk_toggle_school_subjects_status) // PUT /school/subjects/bulk/toggle-status - Bulk toggle subject status in school
            .service(get_school_subjects_by_ids) // POST /school/subjects/bulk/get - Get subjects by multiple IDs in school
            .service(check_school_existing_identifiers), // POST /school/subjects/bulk/check-identifiers - Check identifiers in school
    );
}
