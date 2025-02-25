// use std::sync::Arc;

// use actix_cors::Cors;
// use actix_web::{web, App, HttpServer};
// use config::application_conf::AppConfig;
// use dotenv::dotenv;
// use libs::db::conn_db::ConnDb;
// use slog::info;

// mod config;
// mod controllers;
// mod error;
// mod handlers;
// mod libs;
// mod middleware;
// mod models;
// mod routers;
// mod services;

// use crate::routers::all_routers::all_routers;
// #[derive(Debug)]
// pub struct AppState {
//     pub db: ConnDb,
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     let config = AppConfig::from_env().unwrap();
//     let log = AppConfig::configure_log();

//     let app_state = Arc::new(AppState {
//         db: ConnDb::init().await.unwrap(),
//     });

//     info!(
//         log,
//         "Server is running at http:{}:{}", config.server.host, config.server.port
//     );
//     println!(
//         "Server started at http://{}:{}",
//         config.server.host, config.server.port
//     );

//     HttpServer::new(move || {
//         App::new()
//             .wrap(
//                 Cors::default()
//                     .allow_any_origin()
//                     .allow_any_method()
//                     .allow_any_header(),
//             )
//             .app_data(web::Data::from(app_state.clone()))
//             .configure(|cfg| all_routers(cfg, app_state.clone())) // Configure with all_routers
//     })
//     .bind(format!("{}:{}", config.server.host, config.server.port))?
//     .run()
//     .await
// }

// use std::str::FromStr;

// use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
// use serde_json::to_string_pretty; // Import for proper JSON formatting

// pub fn convert_fields_to_string(mut doc: Document) -> Document {
//     for (key, value) in doc.clone().into_iter() {
//         if let Bson::ObjectId(object_id) = value {
//             doc.insert(key, Bson::String(object_id.to_hex())); // Ensure it converts correctly
//         } else if let Bson::Document(sub_doc) = value {
//             doc.insert(key, Bson::Document(convert_fields_to_string(sub_doc)));
//         } else if let Bson::Array(arr) = value {
//             let converted_arr: Vec<Bson> = arr
//                 .into_iter()
//                 .map(|item| {
//                     if let Bson::ObjectId(object_id) = item {
//                         Bson::String(object_id.to_hex()) // Ensure proper conversion
//                     } else if let Bson::Document(sub_doc) = item {
//                         Bson::Document(convert_fields_to_string(sub_doc))
//                     } else {
//                         item
//                     }
//                 })
//                 .collect();
//             doc.insert(key, Bson::Array(converted_arr));
//         }
//     }
//     doc
// }

// fn main() {
//     let id = ObjectId::from_str("67bd7c03eb09dee0344f5682").unwrap();
//     let sample_doc = doc! {
//         "_id": ObjectId::new(),
//         "name": "Test Name",
//         "class_id": ObjectId::new(),
//         "nested": { "nested_id": ObjectId::new() , "nested_text": "Nested Text" , "id🍀🍀" : id},
//         "array": [ObjectId::new(), "text", 123, id],
//     };

//     let converted_doc = convert_fields_to_string(sample_doc);

//     // Convert to JSON for proper output
//     let json_output = to_string_pretty(&converted_doc).unwrap();
//     println!("Converted Document: {}", json_output);
//     println!("not converted Document: {:?}", sample_doc);
// }

use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use serde_json::to_string_pretty; // Import for proper JSON formatting

pub fn convert_fields_to_string(mut doc: Document) -> Document {
    for (key, value) in doc.clone().into_iter() {
        if let Bson::ObjectId(object_id) = value {
            doc.insert(key, Bson::String(object_id.to_string()));
        } else if let Bson::Document(sub_doc) = value {
            doc.insert(key, Bson::Document(convert_fields_to_string(sub_doc)));
        } else if let Bson::Array(arr) = value {
            let converted_arr: Vec<Bson> = arr
                .into_iter()
                .map(|item| {
                    if let Bson::ObjectId(object_id) = item {
                        Bson::String(object_id.to_string())
                    } else if let Bson::Document(sub_doc) = item {
                        Bson::Document(convert_fields_to_string(sub_doc))
                    } else {
                        item
                    }
                })
                .collect();
            doc.insert(key, Bson::Array(converted_arr));
        }
    }
    doc
}

fn main() {
    let sample_doc = doc! {
        "_id": ObjectId::new(),
        "name": "Test Name",
        "class_id": ObjectId::new(),
        "nested": { "nested_id": ObjectId::new(), "nested_text": "Nested Text" },
        "array": [ObjectId::new(), "text", 123]
    };

    let converted_doc = convert_fields_to_string(sample_doc.clone());

    // Convert to JSON for proper output
    let json_output_original = to_string_pretty(&sample_doc).unwrap();
    let json_output_converted = to_string_pretty(&converted_doc).unwrap();

    println!(
        "Original Document (Not Converted): {}",
        json_output_original
    );
    println!("Converted Document: {}", json_output_converted);
}
