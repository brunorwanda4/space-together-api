use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    config::state::AppState, models::request_error_model::ReqErrModel,
    services::database_status_service::get_database_stats,
};

#[get("/status")]
pub async fn db_status(_: web::Data<AppState>) -> impl Responder {
    let service = get_database_stats().await;

    match service {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/database").service(db_status));
}
