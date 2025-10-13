use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        trade::{Trade, UpdateTrade},
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::{sector_repo::SectorRepo, trade_repo::TradeRepo},
    services::event_service::EventService,
    services::{sector_service::SectorService, trade_service::TradeService},
};

#[get("")]
async fn get_all_trades(state: web::Data<AppState>) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    match service.get_all_trades().await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/others")]
async fn get_all_trades_with_others(state: web::Data<AppState>) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    match service.get_all_trades_with_others().await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_trade_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
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
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    let username = path.into_inner();

    match service.get_trade_by_username(&username).await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/others/{id}")]
async fn get_trade_by_id_with_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    let trade_id = IdType::from_string(path.into_inner());

    match service.get_trade_by_id_with_others(&trade_id).await {
        Ok(trade_with_sector) => HttpResponse::Ok().json(trade_with_sector),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/others/{username}")]
async fn get_trades_by_username_with_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    let username = path.into_inner();

    match service.get_trade_by_username_with_sector(&username).await {
        Ok(trade_with_sector) => HttpResponse::Ok().json(trade_with_sector),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

// NEW ENDPOINTS ADDED HERE:

#[get("/sector/{id}")]
async fn get_trades_by_sector_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    let sector_id = IdType::from_string(path.into_inner());

    match service.get_trades_by_sector_id(&sector_id).await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/trade/{id}")]
async fn get_trades_by_trade_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    let trade_id = IdType::from_string(path.into_inner());

    match service.get_trades_by_trade_id(&trade_id).await {
        Ok(trades) => HttpResponse::Ok().json(trades),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_trade(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Trade>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);

    let sector_repo = SectorRepo::new(&state.db.main_db());
    let sector_service = SectorService::new(&sector_repo);

    match trade_service
        .create_trade(data.into_inner(), &sector_service)
        .await
    {
        Ok(trade) => {
            // 🔔 Broadcast real-time event
            let trade_clone = trade.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = trade_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "trade",
                        &id.to_hex(),
                        &trade_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(trade)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_trade(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateTrade>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let trade_id = IdType::from_string(path.into_inner());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);

    let sector_repo = SectorRepo::new(&state.db.main_db());
    let sector_service = SectorService::new(&sector_repo);

    match trade_service
        .update_trade(&trade_id, data.into_inner(), &sector_service)
        .await
    {
        Ok(trade) => {
            // 🔔 Broadcast real-time event
            let trade_clone = trade.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = trade_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "trade",
                        &id.to_hex(),
                        &trade_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(trade)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_trade(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let trade_id = IdType::from_string(path.into_inner());
    let repo = TradeRepo::new(&state.db.main_db());
    let service = TradeService::new(&repo);

    // Get trade before deletion for broadcasting
    let trade_before_delete = repo.find_by_id(&trade_id).await.ok().flatten();

    match service.delete_trade(&trade_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(trade) = trade_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = trade.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "trade",
                            &id.to_hex(),
                            &trade,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Trade deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trades")
            // Public routes
            .service(get_all_trades) // GET /trades
            .service(get_all_trades_with_others) // GET /trades/others
            .service(get_trades_by_username_with_others) // GET /trades/username/others/{username}
            .service(get_trade_by_id_with_others) // GET /trades/others/{id}
            .service(get_trades_by_username) // GET /trades/username/{username}
            .service(get_trade_by_id) // GET /trades/{id}
            .service(get_trades_by_sector_id) // GET /trades/sector/{id}
            .service(get_trades_by_trade_id) // GET /trades/trade/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_trade) // POST /trades
            .service(update_trade) // PUT /trades/{id}
            .service(delete_trade), // DELETE /trades/{id}
    );
}
