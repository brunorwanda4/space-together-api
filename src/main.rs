#![allow(unused)]

use std::sync::Arc;

use handlers::users_handler::{self, routes};
use axum::{extract::{Path, Query}, response::{Html, IntoResponse}, routing::get, Router};
use libs::db::{self, Database};
use serde::Deserialize;
use tokio::net::TcpListener;

mod handlers;
mod errors;
mod models;
mod libs;


#[derive(Deserialize)]
struct HelloParams {
    name: Option<String>,
}

pub struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {
    let db = Database::init().await;
    let mc = Arc::new(AppState {db : db.expect("Some thing went wrong") });
    
    // ----------- routes ------------
    let user_routers_api = users_handler::routes(mc.clone());

    let routes = Router::new()
        .route("/", get(hello_world))
        .route("/hello", get(hello_1))
        .route("/you/:name", get(hello_2))
        .nest("/api/v1/user", user_routers_api);
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    println!("->> LISTENING on {:?}\n", listener.local_addr());

    axum::serve(listener, routes.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> impl IntoResponse {
    Html("Hello World".to_string())
}

// hello name hello/name=world
async fn hello_1(Query(params): Query<HelloParams>) -> impl IntoResponse {
    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello, {}!", name))
}

// hello name /hello/name
async fn hello_2 (Path(name) : Path<String>) -> impl IntoResponse {
    Html(format!("Hello {} nice to meet you", name ))
}