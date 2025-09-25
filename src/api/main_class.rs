use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::{
        auth_user::AuthUserDto,
        main_class::{MainClass, UpdateMainClass},
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::{main_class_repo::MainClassRepo, trade_repo::TradeRepo},
    services::{main_class_service::MainClassService, trade_service::TradeService},
};

#[get("/trade")]
async fn get_all_main_classes_with_trade(db: web::Data<Database>) -> impl Responder {
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.get_all_with_trade().await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("")]
async fn get_all_main_classes(db: web::Data<Database>) -> impl Responder {
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.get_all().await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_main_class_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let id = IdType::from_string(path.into_inner());

    match service.get_by_id(&id).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_main_class_by_username(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let username = path.into_inner();

    match service.get_by_username(&username).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/others/{username}")]
async fn get_main_class_by_id_with_others(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let main_class_id = IdType::from_string(path.into_inner());

    match service.get_by_id_with_others(&main_class_id).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/others/{username}")]
async fn get_main_class_by_username_with_others(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let username = path.into_inner();

    match service.get_by_username_with_others(&username).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_main_class(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<MainClass>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.create_main_class(data.into_inner()).await {
        Ok(item) => HttpResponse::Created().json(item),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_main_class(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateMainClass>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.update(&id, data.into_inner()).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_main_class(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let repo = MainClassRepo::new(db.get_ref());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.delete(&id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "MainClass deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/main-classes")
            // Public routes
            .service(get_all_main_classes)
            .service(get_all_main_classes_with_trade)
            .service(get_main_class_by_username)
            .service(get_main_class_by_username_with_others)
            .service(get_main_class_by_id_with_others)
            .service(get_main_class_by_id)
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_main_class)
            .service(update_main_class)
            .service(delete_main_class),
    );
}
