use std::sync::Arc;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use database::database_conn::DBConn;
use errors::MyError;
use routes::all_routes::all_routes;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;

mod controller;
mod database;
mod error;
mod errors;
mod handlers;
mod libs;
mod models;
mod routes;

pub struct AppState {
    db: DBConn,
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let db = DBConn::init()
        .await
        .expect("Can not connect to database after initialization");
    let mc = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let routes = all_routes(mc).await;

    let app = routes.layer(cors).layer(CookieManagerLayer::new());
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    println!("->> LISTENING on {:?}\n", listener.local_addr());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
