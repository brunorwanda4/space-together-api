mod api;
mod config;
mod domain;
mod errors;
mod guards;
mod helpers;
mod mappers;
mod middleware;
mod models;
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

    let port = env::var("PORT").unwrap_or("4666".to_string());
    let db = config::db::init_db().await;

    let state = web::Data::new(config::state::AppState::new(db));

    println!("🚀 Space-Together backend running on http://127.0.0.1:{port}");
    println!("📡 Real-time events: http://127.0.0.1:{port}/events/stream");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(state.clone()) // Use the unified app state
            .configure(api::init_routes)
    })
    .bind(("127.0.0.1", port.parse::<u16>().unwrap()))?
    .run()
    .await
}
