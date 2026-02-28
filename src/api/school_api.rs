use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        school::{School, SchoolAcademicRequest, SchoolPartial},
    },
    errors::AppError,
    models::{api_request_model::RequestQuery, id_model::IdType},
    repositories::user_repo::UserRepo,
    services::{
        event_service::EventService, school_service::SchoolService, user_service::UserService,
    },
    utils::api_utils::build_extra_match,
};

#[get("")]
async fn get_all_schools(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_school_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolService::new(&state.db.main_db());

    match service.find_one(Some(&id), None).await {
        Ok(school) => HttpResponse::Ok().json(school),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_school_by_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(school) => HttpResponse::Ok().json(school),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_school(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<School>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_or_staff(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = SchoolService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(school) => {
            let clone = school.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "school",
                        &id.to_hex(),
                        None,
                        &clone,
                    )
                    .await;
                }
            });

            let school_id = match school.id {
                Some(id) => IdType::ObjectId(id),
                None => {
                    return HttpResponse::BadRequest().json(AppError {
                        message: "Failed to get school id".into(),
                    })
                }
            };

            let user_repo = UserRepo::new(&state.db.main_db());
            let user_service = UserService::new(&user_repo);

            let user_id = IdType::from_string(&logged_user.id);

            match user_service.add_school_to_user(&user_id, &school_id).await {
                Ok(_) => (),
                Err(err) => {
                    return HttpResponse::BadRequest().json(err);
                }
            }

            let token = match service
                .create_school_token(&school_id, &logged_user, &state)
                .await
            {
                Ok(token) => token,
                Err(err) => {
                    return HttpResponse::BadRequest().json(err);
                }
            };

            let mut school_json = serde_json::to_value(&school).unwrap_or_default();

            if let serde_json::Value::Object(ref mut obj) = school_json {
                obj.insert("token".to_string(), serde_json::json!(token));
            }

            HttpResponse::Created().json(school_json)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_school(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SchoolPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolService::new(&state.db.main_db());

    match service.update(&id, &data.into_inner()).await {
        Ok(school) => {
            let clone = school.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "school",
                        &id.to_hex(),
                        None,
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(school)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_school(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = SchoolService::new(&state.db.main_db());

    match service.delete(&id).await {
        Ok(school) => {
            let clone = school.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "school",
                        &id.to_hex(),
                        None,
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(school)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_schools(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = SchoolService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}/search/members")]
async fn search_school_members(
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let school_id = path.into_inner();

    // Get school to verify it exists and get database name
    let school_service = SchoolService::new(&state.db.main_db());
    let school = match school_service
        .find_one(Some(&IdType::from_string(&school_id)), None)
        .await
    {
        Ok(school) => school,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    let school_db_name = match school.database_name {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest().json(AppError {
                message: "School database not configured".to_string(),
            })
        }
    };

    let db = state.db.get_db(&school_db_name);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match school_service
        .search_members(&db, query.filter.clone(), query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/refresh-school-token")]
async fn refresh_school_token(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let school_service = SchoolService::new(&state.db.main_db());

    // Extract raw School-Token header
    let token = match req.headers().get("School-Token") {
        Some(hv) => match hv.to_str() {
            Ok(s) => s.trim().to_string(),
            Err(_) => {
                return HttpResponse::Unauthorized().json(AppError {
                    message: "Invalid School-Token header format".to_string(),
                })
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(AppError {
                message: "Missing School-Token header".to_string(),
            })
        }
    };

    match school_service.refresh_school_token(&token).await {
        Ok(new_token) => HttpResponse::Ok().json(serde_json::json!({
            "schoolAccessToken": new_token
        })),
        Err(error) => HttpResponse::Unauthorized().json(error),
    }
}

#[post("/{id}/academics")]
async fn setup_school_academics(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SchoolAcademicRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    let target_school_id_str = path.into_inner();

    if let Err(err) =
        crate::guards::role_guard::check_school_access(&logged_user, &target_school_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = SchoolService::new(&state.db.main_db());
    let target_school_id = IdType::from_string(target_school_id_str);

    match service
        .setup_school_academics(
            &target_school_id,
            data.into_inner(),
            state.clone(),
            logged_user,
        )
        .await
    {
        Ok(response) => {
            // Broadcast event for academic setup completion
            let state_clone = state.clone();
            let school_id_hex = match &target_school_id {
                IdType::ObjectId(id) => id.to_hex(),
                IdType::String(id) => id.clone(),
            };

            actix_rt::spawn(async move {
                EventService::broadcast_updated(
                    &state_clone,
                    "school_academics",
                    &school_id_hex.clone(),
                    None,
                    &serde_json::json!({
                        "school_id": school_id_hex,
                        "created_classes": response.created_classes,
                        "created_subjects": response.created_subjects,
                        "success": response.success
                    }),
                )
                .await;
            });

            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::BadRequest().json(e),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/schools")
            .service(get_all_schools)
            .service(get_school_by_match)
            .service(count_schools)
            .service(get_school_by_id)
            .service(search_school_members)
            .service(refresh_school_token)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_school)
            .service(update_school)
            .service(delete_school)
            .service(setup_school_academics),
    );
}
