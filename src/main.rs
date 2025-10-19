mod api;
mod config;
mod controller;
mod domain;
mod errors;
mod guards;
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
        let address = ("127.0.0.1", port_num);

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
}
