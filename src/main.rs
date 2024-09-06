// #![allow(unused)]

use std::sync::Arc;

use errors::MyError;
use axum::{extract::{Path, Query}, http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method}, response::{Html, IntoResponse}, routing::get, Router};
use libs::db::{self, Database};
use routes::all_routers;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;

mod handlers;
mod errors;
mod models;
mod libs;

mod routes;

#[derive(Deserialize)]
struct HelloParams {
    name: Option<String>,
}

pub struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() -> Result<() , MyError> {
    let db = Database::init().await?;
    let mc = Arc::new(AppState {db : db});

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
    
    let routes = all_routers(mc).await;

    let app = routes
     .layer(cors)
     .layer(CookieManagerLayer::new());
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    println!("->> LISTENING on {:?}\n", listener.local_addr());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// async fn hello_world() -> impl IntoResponse {
//     Html("Hello World".to_string())
// }

// hello name hello/name=world
// async fn hello_1(Query(params): Query<HelloParams>) -> impl IntoResponse {
//     let name = params.name.as_deref().unwrap_or("World");
//     Html(format!("Hello, {}!", name))
// }

// hello name /hello/name
// async fn hello_2 (Path(name) : Path<String>) -> impl IntoResponse {
//     Html(format!("Hello {} nice to meet you", name ))
// }