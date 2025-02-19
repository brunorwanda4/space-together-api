use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use config::application_conf::AppConfig;
use dotenv::dotenv;
use libs::db::conn_db::ConnDb;
use slog::info;

mod config;
mod controllers;
mod error;
mod handlers;
mod libs;
mod middleware;
mod models;
mod routers;
mod services;

use crate::routers::all_routers::all_routers;
#[derive(Debug)]
pub struct AppState {
    pub db: ConnDb,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = AppConfig::from_env().unwrap();
    let log = AppConfig::configure_log();

    let app_state = Arc::new(AppState {
        db: ConnDb::init().await.unwrap(),
    });

    info!(
        log,
        "Server is running at http:{}:{}", config.server.host, config.server.port
    );
    println!(
        "Server started at http://{}:{}",
        config.server.host, config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::from(app_state.clone()))
            .configure(|cfg| all_routers(cfg, app_state.clone())) // Configure with all_routers
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
