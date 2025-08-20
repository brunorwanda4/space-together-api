use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use config::application_conf::AppConfig;
use dotenv::dotenv;
use libs::db::conn_db::ConnDb;
use log::{error, info};
use routers::all_routers::all_routers;

mod config;
mod controllers;
mod error;
mod handlers;
mod libs;
mod middleware;
mod models;
mod routers;
mod services;

#[derive(Debug)]
pub struct AppState {
    pub db: ConnDb,
}

/// Application entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment and logging
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Loading application configuration...");

    // Load configuration with proper error handling
    let config = AppConfig::from_env().unwrap_or_else(|e| {
        error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    info!("Initializing database connection...");

    // Initialize database connection with error handling
    let db_conn = ConnDb::init().await.unwrap_or_else(|e| {
        error!("Failed to initialize database: {}", e);
        std::process::exit(1);
    });

    // Create shared application state
    let app_state = Arc::new(AppState { db: db_conn });

    // Convert port to u16 (actix-web requirement)
    let port = config.server.port as u16;

    info!("Server starting on {}:{}", config.server.host, port);

    // Determine worker count (use default based on CPU count)
    let worker_count = num_cpus::get();
    info!("Starting server with {} workers", worker_count);

    // Create HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::from(Arc::clone(&app_state)))
            .configure(|cfg| all_routers(cfg, Arc::clone(&app_state)))
    })
    .bind((config.server.host.as_str(), port))?
    .workers(worker_count)
    .shutdown_timeout(30) // 30 seconds graceful shutdown timeout
    .run()
    .await
}
