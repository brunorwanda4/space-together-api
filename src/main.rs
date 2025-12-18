mod api;
mod config;
mod controller;
mod domain;
mod errors;
mod guards;
mod handler;
mod helpers;
mod mappers;
mod middleware;
mod models;
mod pipeline;
mod repositories;
mod services;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    config::logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "4646".to_string());
    let address = format!("0.0.0.0:{port}");

    let mongo_manager = config::db::init_mongo_manager().await;
    let state = web::Data::new(config::state::AppState::new(mongo_manager.clone()));

    println!("🚀 Space-Together backend starting on {address}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(crate::middleware::tenant_middleware::TenantMiddleware::new(
                mongo_manager.clone(),
            ))
            .app_data(state.clone())
            .configure(api::init_routes)
    })
    .bind(address)?
    .run()
    .await
}
