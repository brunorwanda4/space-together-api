use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::user::User, models::id_model::IdType, models::request_error_model::ReqErrModel,
    repositories::user_repo::UserRepo, services::user_service::UserService,
};

#[post("/users")]
async fn create_user(data: web::Json<User>, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    let new_user = data.into_inner();

    match service.create_user(new_user).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

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

#[put("/users/{id}")]
async fn update_user(
    path: web::Path<String>,
    data: web::Json<User>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    let user_id = IdType::from_string(path.into_inner());
    let updated_data = data.into_inner();

    match service.update_user(&user_id, updated_data).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/users/{id}")]
async fn delete_user(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = UserRepo::new(db.get_ref());
    let service = UserService::new(&repo);

    let user_id = IdType::from_string(path.into_inner());

    match service.delete_user(&user_id).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({"message": "User deleted successfully"}))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_users);
    cfg.service(get_user_by_id);
    cfg.service(get_user_by_username);
    cfg.service(update_user);
    cfg.service(delete_user);
    cfg.service(create_user);
}
