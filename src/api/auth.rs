use actix_web::{get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth::{LoginUser, RegisterUser},
        auth_user::AuthUserDto,
        user::UpdateUserDto,
    },
    middleware::jwt_middleware::JwtMiddleware,
    models::request_error_model::ReqErrModel,
    repositories::user_repo::UserRepo,
    services::{auth_service::AuthService, user_service::UserService},
};

#[post("/register")]
async fn register_user(
    data: web::Json<RegisterUser>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = AuthService::new(&repo);

    match service.register(data.into_inner()).await {
        Ok((token, user)) => HttpResponse::Created().json(serde_json::json!({
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "accessToken": token,
            "image": user.image,
            "role": user.role,
            "username": user.username,
            "bio": user.bio,
            "schoolAccessToken": ""
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/login")]
async fn login_user(data: web::Json<LoginUser>, state: web::Data<AppState>) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = AuthService::new(&repo);

    match service.login(data.into_inner()).await {
        Ok((token, user)) => HttpResponse::Ok().json(serde_json::json!({
             "id": user.id,
            "email": user.email,
            "name": user.name,
            "accessToken": token,
            "image": user.image,
            "role": user.role,
            "username": user.username,
            "bio": user.bio,
            "schoolAccessToken": ""
        })),
        Err(message) => HttpResponse::Unauthorized().json(ReqErrModel { message }),
    }
}

#[get("/me")]
async fn get_me(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
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
        Err(message) => HttpResponse::Unauthorized().json(ReqErrModel { message }),
    }
}

#[patch("/onboarding")]
async fn onboarding_user(
    req: HttpRequest,
    data: web::Json<UpdateUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ReqErrModel {
                message: "Unauthorized".to_string(),
            })
        }
    };

    let repo = UserRepo::new(&state.db.main_db());
    let user_service = UserService::new(&repo);
    let auth_service = AuthService::new(&repo);

    match auth_service
        .onboard_user(&logged_user.id, data.into_inner(), &user_service)
        .await
    {
        Ok((new_token, user)) => HttpResponse::Ok().json(serde_json::json!({
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "accessToken": new_token,
            "image": user.image,
            "role": user.role,
            "username": user.username,
            "bio": user.bio,
            "schoolAccessToken": ""
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/refresh")]
async fn refresh_token(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let repo = UserRepo::new(&state.db.main_db());
    let service = AuthService::new(&repo);

    // Extract token from Authorization header
    let token = match req.headers().get("Authorization") {
        Some(hv) => match hv.to_str() {
            Ok(s) => {
                if let Some(stripped) = s.strip_prefix("Bearer ") {
                    stripped.to_string()
                } else {
                    s.to_string()
                }
            }
            Err(_) => {
                return HttpResponse::Unauthorized().json(ReqErrModel {
                    message: "Invalid authorization header".to_string(),
                })
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(ReqErrModel {
                message: "Missing authorization header".to_string(),
            })
        }
    };

    match service.refresh_token(&token).await {
        Ok(new_token) => HttpResponse::Ok().json(serde_json::json!({
            "accessToken": new_token
        })),
        Err(message) => HttpResponse::Unauthorized().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(register_user);
    cfg.service(login_user);
    cfg.service(get_me);

    cfg.service(
        web::scope("/auth")
            .wrap(JwtMiddleware)
            .service(onboarding_user)
            .service(refresh_token),
    );
}
