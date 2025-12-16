<<<<<<< HEAD
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
=======
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
    let mut port_num = port.parse::<u16>().unwrap_or(4646);

    let mongo_manager = config::db::init_mongo_manager().await;
    let state = web::Data::new(config::state::AppState::new(mongo_manager.clone()));

    println!("🚀 Space-Together backend starting...");

    // Unix: systemfd hot-reload
    #[cfg(unix)]
    {
        if let Ok(fd_count) = env::var("LISTEN_FDS") {
            if fd_count != "0" {
                println!("🔁 Hot reload mode enabled via systemfd");
                use std::net::TcpListener;
                use std::os::fd::FromRawFd;

                unsafe {
                    let listener = TcpListener::from_raw_fd(3);
                    println!("🚀 Using socket inherited from systemfd");
                    return HttpServer::new(move || {
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
                    .listen(listener)?
                    .run()
                    .await;
                }
            }
        }
    }

    // Windows / normal mode
    println!("🪟 Running in normal mode (Windows or no systemfd)");

    loop {
        let address = ("0.0.0.0", port_num);

        // Create a new HttpServer with fresh closure each iteration
        let server = HttpServer::new({
            let mongo_manager = mongo_manager.clone();
            let state = state.clone();
            move || {
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
            }
        });

        match server.bind(address) {
            Ok(srv) => {
                println!("✅ Space-Together backend running on http://127.0.0.1:{port_num}");
                println!("📡 Real-time events: http://127.0.0.1:{port_num}/events/stream");
                return srv.run().await;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                println!(
                    "⚠️  Port {port_num} already in use, trying {}",
                    port_num + 1
                );
                port_num += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed to bind server: {e}");
                return Err(e);
            }
        }
    }
>>>>>>> happy/main
}
