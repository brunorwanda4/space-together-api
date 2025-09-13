use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::auth::{LoginUser, RegisterUser},
    models::request_error_model::ReqErrModel,
    repositories::user_repo::UserRepo,
    services::auth_service::AuthService,
};

#[post("/register")]
async fn register_user(data: web::Json<RegisterUser>, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = AuthService::new(&repo);

    match service.register(data.into_inner()).await {
        Ok((token, user)) => HttpResponse::Created().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user.id,
                "name": user.name,
                "email": user.email,
                "role": user.role,
                "username": user.username
            }
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/login")]
async fn login_user(data: web::Json<LoginUser>, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = AuthService::new(&repo);

    match service.login(data.into_inner()).await {
        Ok((token, user)) => HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": {
                "id": user.id,
                "name": user.name,
                "email": user.email,
                "role": user.role,
                "username": user.username
            }
        })),
        Err(message) => HttpResponse::Unauthorized().json(ReqErrModel { message }),
    }
}

#[get("/me")]
async fn get_me(req: HttpRequest, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = AuthService::new(&repo);

    let token = match req.headers().get("Authorization") {
        Some(hv) => match hv.to_str() {
            Ok(s) => {
                let s = s.trim();
                if let Some(stripped) = s.strip_prefix("Bearer ") {
                    stripped.to_string()
                } else {
                    s.to_string()
                }
            }
            Err(_) => return HttpResponse::Unauthorized().body("Invalid authorization header"),
        },
        None => return HttpResponse::Unauthorized().body("Missing authorization header"),
    };

    match service.get_user_from_token(&token).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(msg) => HttpResponse::Unauthorized().body(msg),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(register_user);
    cfg.service(login_user);
    cfg.service(get_me);
}
