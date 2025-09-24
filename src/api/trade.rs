use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::{
        auth_user::AuthUserDto,
        trade::{Trade, UpdateTrade},
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::{sector_repo::SectorRepo, trade_repo::TradeRepo},
    services::{sector_service::SectorService, trade_service::TradeService},
};

#[get("")]
async fn get_all_trades(db: web::Data<Database>) -> impl Responder {
    let repo = TradeRepo::new(db.get_ref());
    let service = TradeService::new(&repo);

    match service.get_all_trades().await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/trades-with-sector")]
async fn get_all_trades_with_sector(db: web::Data<Database>) -> impl Responder {
    let repo = TradeRepo::new(db.get_ref());
    let service = TradeService::new(&repo);

    match service.get_all_trades_with_sector().await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_trade_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = TradeRepo::new(db.get_ref());
    let service = TradeService::new(&repo);

    let trade_id = IdType::from_string(path.into_inner());

    match service.get_trade_by_id(&trade_id).await {
        Ok(trade) => HttpResponse::Ok().json(trade),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_trades_by_username(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = TradeRepo::new(db.get_ref());
    let service = TradeService::new(&repo);

    let username = path.into_inner();

    match service.get_trade_by_username(&username).await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_trade(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Trade>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);

    let sector_repo = SectorRepo::new(db.get_ref());
    let sector_service = SectorService::new(&sector_repo);

    match trade_service
        .create_trade(data.into_inner(), &sector_service)
        .await
    {
        Ok(trade) => HttpResponse::Created().json(trade),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_trade(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateTrade>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let trade_id = IdType::from_string(path.into_inner());
    let trade_repo = TradeRepo::new(db.get_ref());
    let trade_service = TradeService::new(&trade_repo);

    let sector_repo = SectorRepo::new(db.get_ref());
    let sector_service = SectorService::new(&sector_repo);

    match trade_service
        .update_trade(&trade_id, data.into_inner(), &sector_service)
        .await
    {
        Ok(trade) => HttpResponse::Ok().json(trade),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_trade(
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

    let trade_id = IdType::from_string(path.into_inner());
    let repo = TradeRepo::new(db.get_ref());
    let service = TradeService::new(&repo);

    match service.delete_trade(&trade_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Trade deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trades")
            // Public routes
            .service(get_all_trades)
            .service(get_trades_by_username)
            .service(get_all_trades_with_sector)
            .service(get_trade_by_id)
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_trade)
            .service(update_trade)
            .service(delete_trade),
    );
}
