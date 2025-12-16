use std::env;

use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    config::state::AppState, models::request_error_model::ReqErrModel,
    services::database_status_service::get_database_stats,
};

#[get("/status")]
pub async fn db_status(_: web::Data<AppState>) -> impl Responder {
    let main_db_name = env::var("MAIN_DB_NAME").unwrap_or_else(|_| "space_together".to_string());
    let service = get_database_stats(&main_db_name).await;

    match service {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}")]
pub async fn school_db_status(path: web::Path<String>) -> impl Responder {
    let school_id = path.into_inner();

    // Map school_id to DB name
    let db_name = format!("school_{}", school_id);

    match get_database_stats(&db_name).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/database")
            .service(db_status)
            .service(school_db_status),
    );
}
