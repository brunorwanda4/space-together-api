// use mongodb::bson::{doc, Bson, Document};
// use serde_json::to_string_pretty;

// pub fn convert_fields_to_string(doc: Document) -> Document {
//     let mut new_doc = Document::new();

//     for (key, value) in doc {
//         let new_value = match value {
//             Bson::Document(sub_doc) => {
//                 // Check if it's an embedded ObjectId format like { "$oid": "some_id" }
//                 if sub_doc.len() == 1 {
//                     if let Some(Bson::String(oid)) = sub_doc.get("$oid") {
//                         Bson::String(oid.clone()) // Convert { "$oid": "xyz" } → "xyz"
//                     } else {
//                         Bson::Document(convert_fields_to_string(sub_doc))
//                     }
//                 } else {
//                     Bson::Document(convert_fields_to_string(sub_doc))
//                 }
//             }
//             Bson::Array(arr) => {
//                 let converted_arr: Vec<Bson> = arr
//                     .into_iter()
//                     .map(|item| match item {
//                         Bson::Document(sub_doc) => {
//                             if sub_doc.len() == 1 {
//                                 if let Some(Bson::String(oid)) = sub_doc.get("$oid") {
//                                     Bson::String(oid.clone())
//                                 } else {
//                                     Bson::Document(convert_fields_to_string(sub_doc))
//                                 }
//                             } else {
//                                 Bson::Document(convert_fields_to_string(sub_doc))
//                             }
//                         }
//                         other => other,
//                     })
//                     .collect();
//                 Bson::Array(converted_arr)
//             }
//             _ => value,
//         };
//         new_doc.insert(key, new_value);
//     }

//     new_doc
// }

// fn main() {
//     let sample_doc = doc! {
//         "_id": { "$oid": "67bd9675711fb75f575531c6" },
//         "name": "Add your name in the body",
//         "class_room_id": null,
//         "class_id": { "$oid": "67b741d5c87ed6aa9e5b6efd" },
//         "code": "1232jf",
//         "sector_id": [{ "$oid": "67b741d5c87ed6aa9e5b6efd" },{ "$oid": "67b741d5c87ed6aa9e5b6efd" },{ "$oid": "67b741d5c87ed6aa9e5b6efd" },{ "$oid": "67b741d5c87ed6aa9e5b6efd" }],
//         "trade_id": null,
//         "subject_type": null,
//         "curriculum": null,
//         "copyright": null,
//         "learning_hours": null,
//         "issue_date": null,
//         "purpose": null,
//         "symbol": null,
//         "knowledge": null,
//         "skills": null,
//         "attitude": null,
//         "resource": null,
//         "competence": null,
//         "created_at": "2025-02-25T10:07:49.259Z",
//         "updated_at": null
//     };

//     let converted_doc = convert_fields_to_string(sample_doc);
//     let json_output_converted = to_string_pretty(&converted_doc).unwrap();

//     println!("Converted Document: {}", json_output_converted);
// }

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use config::application_conf::AppConfig;
use dotenv::dotenv;
use libs::db::conn_db::ConnDb;

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
    let app_state = Arc::new(AppState {
        db: ConnDb::init().await.unwrap(),
    });

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
