mod api;
mod config;
mod domain;
mod errors;
mod mappers;
mod models;
mod repositories;
mod services;
mod utils;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    config::logger::init();

    let port = env::var("PORT").unwrap_or("4666".to_string());
    let db = config::db::init_db().await;

    println!("🚀 Space-Together backend running on http://127.0.0.1:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(api::init_routes)
    })
    .bind(("127.0.0.1", port.parse::<u16>().unwrap()))?
    .run()
    .await
}
