use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::{
        auth_user::AuthUserDto,
        user::{UpdateUserDto, User},
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::user_repo::UserRepo,
    services::user_service::UserService,
};

#[get("/users")]
async fn get_all_users(db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    match service.get_all_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/users/{id}")]
async fn get_user_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    let user_id = IdType::from_string(path.into_inner());

    match service.get_user_by_id(&user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/users/username/{username}")]
async fn get_user_by_username(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    let username = path.into_inner();

    match service.get_user_by_username(&username).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("/")]
async fn create_user(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<User>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    match service.create_user(data.into_inner()).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_user(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateUserDto>,
    db: web::Data<Database>,
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
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    match service
        .update_user(&target_user_id, data.into_inner())
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_user(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    db: web::Data<Database>,
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

    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    match service.delete_user(&target_user_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "User deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    // Public routes
    cfg.service(get_all_users);
    cfg.service(get_user_by_id);
    cfg.service(get_user_by_username);

    // Protected routes (JWT required)
    cfg.service(
        web::scope("/users")
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_user)
            .service(update_user)
            .service(delete_user),
    );
}
