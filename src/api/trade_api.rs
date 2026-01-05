use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        trade::{Trade, TradePartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, trade_service::TradeService},
    utils::api_utils::build_extra_match,
};

/// ------------------------------------------------------
/// GET /trades
/// ------------------------------------------------------
#[get("")]
async fn get_all_trades(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TradeService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
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

/// ------------------------------------------------------
/// GET /trades/with-relations
/// ------------------------------------------------------
#[get("/others")]
async fn get_all_trades_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TradeService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /trades/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_trade_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TradeService::new(&state.db.main_db());

    match service.find_one(Some(&id), None).await {
        Ok(trade) => HttpResponse::Ok().json(trade),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /trades/{id}/with-relations
/// ------------------------------------------------------
#[get("/{id}/others")]
async fn get_trade_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TradeService::new(&state.db.main_db());

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(trade) => HttpResponse::Ok().json(trade),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /trades/match
/// ------------------------------------------------------
#[get("/match")]
async fn get_trade_by_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TradeService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(trade) => HttpResponse::Ok().json(trade),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// POST /trades
/// ------------------------------------------------------
#[post("")]
async fn create_trade(
    _user: web::ReqData<AuthUserDto>,
    data: web::Json<Trade>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TradeService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(trade) => {
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
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /trades/{id}
/// ------------------------------------------------------
#[put("/{id}")]
async fn update_trade(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<TradePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TradeService::new(&state.db.main_db());

    match service.update(&id, &data.into_inner()).await {
        Ok(trade) => {
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
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// DELETE /trades/{id}
/// ------------------------------------------------------
#[delete("/{id}")]
async fn delete_trade(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = TradeService::new(&state.db.main_db());

    match service.delete(&id).await {
        Ok(trade) => {
            let trade_clone = trade.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = trade_clone.id {
                    EventService::broadcast_deleted(
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
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /trades/count
/// ------------------------------------------------------
#[get("/count")]
async fn count_trades(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = TradeService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// INIT
/// ------------------------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trades")
            .service(get_all_trades)
            .service(get_all_trades_with_relations)
            .service(get_trade_by_match)
            .service(count_trades)
            .service(get_trade_by_id)
            .service(get_trade_by_id_with_relations)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_trade)
            .service(update_trade)
            .service(delete_trade),
    );
}
