use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        school::{School, UpdateSchool},
    },
    models::{api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel},
    repositories::school_repo::SchoolRepo,
    services::{
        event_service::EventService, school_service::SchoolService, tenant_service::TenantService,
    },
};

#[get("")]
async fn get_all_schools(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    match service
        .get_all_schools(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(schools) => HttpResponse::Ok().json(schools),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats")]
async fn get_school_stats(state: web::Data<AppState>) -> impl Responder {
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    match service.get_school_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_school_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    let school_id = IdType::from_string(path.into_inner());

    match service.get_school_by_id(&school_id).await {
        Ok(school) => HttpResponse::Ok().json(school),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_school_by_username(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    let username = path.into_inner();

    match service.get_school_by_username(&username).await {
        Ok(school) => HttpResponse::Ok().json(school),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_school_by_code(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    let code = path.into_inner();

    match service.get_school_by_code(&code).await {
        Ok(school) => HttpResponse::Ok().json(school),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}
#[post("")]
async fn create_school(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<School>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // ✅ Only admin or staff can create schools
    if let Err(err) = crate::guards::role_guard::check_admin_or_staff(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    // ✅ Initialize repo and service
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    // ✅ Create school record in main DB
    match service.create_school(data.into_inner()).await {
        Ok(mut school) => {
            if let Some(ref id) = school.id {
                // ✅ Generate DB name like school_<id>
                let school_id_hex = id.to_hex();
                let db_name = state.db.school_db_name_from_id(&school_id_hex);

                // ✅ Initialize tenant DB (collections, indexes, seeds)
                let mongo_clone = state.db.clone();
                let db_name_clone = db_name.clone();

                if let Err(e) =
                    TenantService::initialize_school_db(&mongo_clone, &db_name_clone).await
                {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "message": format!("Failed to initialize school DB: {}", e)
                    }));
                }

                // ✅ Update main DB record with the database_name
                let update_result = service
                    .update_school(
                        &IdType::ObjectId(*id),
                        UpdateSchool {
                            database_name: Some(db_name_clone.clone()),
                            ..Default::default() // ✅ requires UpdateSchool: Default
                        },
                    )
                    .await;

                if let Err(e) = update_result {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "message": format!("Failed to update school with db_name: {}", e)
                    }));
                }

                // ✅ Update local copy to include db_name before returning
                school.database_name = Some(db_name_clone);
            }

            // ✅ Broadcast created event asynchronously
            let school_clone = school.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = school_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "school",
                        &id.to_hex(),
                        &school_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(school)
        }

        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_school(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSchool>,
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

    let target_school_id_str = path.into_inner();

    // Check if user has permission to update school
    if let Err(err) =
        crate::guards::role_guard::check_school_access(&logged_user, &target_school_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_school_id = IdType::from_string(target_school_id_str);
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    match service
        .update_school(&target_school_id, data.into_inner())
        .await
    {
        Ok(school) => {
            // 🔔 Broadcast real-time event
            let school_clone = school.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = school_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "school",
                        &id.to_hex(),
                        &school_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(school)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/partial")]
async fn update_school_partial(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSchool>,
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

    let target_school_id_str = path.into_inner();

    // Check if user has permission to update school
    if let Err(err) =
        crate::guards::role_guard::check_school_access(&logged_user, &target_school_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_school_id = IdType::from_string(target_school_id_str);
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    match service
        .update_school_partial(&target_school_id, data.into_inner())
        .await
    {
        Ok(school) => {
            // 🔔 Broadcast real-time event
            let school_clone = school.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = school_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "school",
                        &id.to_hex(),
                        &school_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(school)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_school(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let target_school_id_str = path.into_inner();

    // Only admin can delete schools
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_school_id = IdType::from_string(target_school_id_str);
    let repo = SchoolRepo::new(&state.db.main_db());
    let service = SchoolService::new(&repo);

    // Get school before deletion for broadcasting
    let school_before_delete = repo.find_by_id(&target_school_id).await.ok().flatten();

    match service.delete_school(&target_school_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(school) = school_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = school.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "school",
                            &id.to_hex(),
                            &school,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "School deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/schools")
            // Public routes (read-only)
            .service(get_school_stats) // GET /schools/stats - Get school statistics and analytics
            .service(get_school_by_username) // GET /schools/username/{username} - Get school by username
            .service(get_school_by_code) // GET /schools/code/{code} - Get school by institutional code
            .service(get_school_by_id) // GET /schools/{id} - Get school by ID
            .service(get_all_schools) // GET /schools - Get all schools with filtering and pagination
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_school) // POST /schools - Create new school (Admin/SchoolStaff only)
            .service(update_school) // PUT /schools/{id} - Full update school (Admin/SchoolStaff only)
            .service(update_school_partial) // PUT /schools/{id}/partial - Partial update school (Admin/SchoolStaff only)
            .service(delete_school), // DELETE /schools/{id} - Delete school (Admin only)
    );
}
