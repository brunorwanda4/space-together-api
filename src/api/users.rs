use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        user::{UpdateUserDto, User},
    },
    models::{api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel},
    repositories::user_repo::UserRepo,
    services::{event_service::EventService, user_service::UserService},
};

#[get("")]
async fn get_all_users(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    match service
        .get_all_users(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats")]
async fn get_user_stats(state: web::Data<AppState>) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    match service.get_user_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_user_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    let user_id = IdType::from_string(path.into_inner());

    match service.get_user_by_id(&user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_user_by_username(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    let username = path.into_inner();

    match service.get_user_by_username(&username).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_user(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<User>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    match service.create_user(data.into_inner()).await {
        Ok(user) => {
            // 🔔 Broadcast real-time event
            let user_clone = user.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = user_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "user",
                        &id.to_hex(),
                        &user_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(user)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_user(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateUserDto>,
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

    let target_user_id_str = path.into_inner();

    if let Err(err) =
        crate::guards::role_guard::check_owner_or_admin(&logged_user, &target_user_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_user_id = IdType::from_string(target_user_id_str);
    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    match service
        .update_user(&target_user_id, data.into_inner())
        .await
    {
        Ok(user) => {
            // 🔔 Broadcast real-time event
            let user_clone = user.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = user_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "user",
                        &id.to_hex(),
                        &user_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(user)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_user(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let target_user_id_str = path.into_inner();

    if let Err(err) =
        crate::guards::role_guard::check_owner_or_admin(&logged_user, &target_user_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_user_id = IdType::from_string(target_user_id_str);
    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    // Get user before deletion for broadcasting
    let user_before_delete = repo.find_by_id(&target_user_id).await.ok().flatten();

    match service.delete_user(&target_user_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(user) = user_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = user.id {
                        EventService::broadcast_deleted(&state_clone, "user", &id.to_hex(), &user)
                            .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "User deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Add a school to a user and set it as current
#[post("/{user_id}/schools/{school_id}")]
async fn add_school_to_user(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    let (user_id_str, school_id_str) = path.into_inner();
    let user_id = IdType::from_string(&user_id_str);
    let school_id = IdType::from_string(&school_id_str);

    if let Err(err) = crate::guards::role_guard::check_owner_or_admin(&logged_user, &user_id_str) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    match service.add_school_to_user(&user_id, &school_id).await {
        Ok(updated_user) => {
            // 🔔 Broadcast update
            let state_clone = state.clone();
            let user_clone = updated_user.clone();
            actix_rt::spawn(async move {
                if let Some(id) = user_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "user",
                        &id.to_hex(),
                        &user_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(updated_user)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Remove a school from a user
#[delete("/{user_id}/schools/{school_id}")]
async fn remove_school_from_user(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    let (user_id_str, school_id_str) = path.into_inner();
    let user_id = IdType::from_string(&user_id_str);
    let school_id = IdType::from_string(&school_id_str);

    if let Err(err) = crate::guards::role_guard::check_owner_or_admin(&logged_user, &user_id_str) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = UserRepo::new(&state.db.main_db());
    let service = UserService::new(&repo);

    match service.remove_school_from_user(&user_id, &school_id).await {
        Ok(updated_user) => {
            // 🔔 Broadcast update
            let state_clone = state.clone();
            let user_clone = updated_user.clone();
            actix_rt::spawn(async move {
                if let Some(id) = user_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "user",
                        &id.to_hex(),
                        &user_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(updated_user)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            // Public routes
            .service(get_user_stats) // GET /users/stats
            .service(get_user_by_username) // GET /users/username/{username}
            .service(get_all_users) // GET /users
            .service(get_user_by_id) // GET /users/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_user) // POST /users
            .service(remove_school_from_user) // DELETE /users/{user_id}/schools/{school_id}
            .service(add_school_to_user) // POST /users/{user_id}/schools/{school_id}
            .service(update_user) // PUT /users/{id}
            .service(delete_user), // DELETE /users/{id}
    );
}
