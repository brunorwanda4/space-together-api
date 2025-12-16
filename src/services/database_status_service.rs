use std::env;

use futures::TryStreamExt;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde_json::to_string;

use crate::{
    domain::database_status::{CollectionStats, DatabaseStats},
    utils::bytes::format_bytes,
};

pub async fn get_database_stats(db_name: &str) -> Result<DatabaseStats, String> {
    let uri = env::var("MONGO_URI").expect("❌ MONGO_URI not set in .env");
    let options = ClientOptions::parse(uri.clone())
        .await
        .map_err(|e| e.to_string())?;
    let client = Client::with_options(options).map_err(|e| e.to_string())?;

    let database = client.database(db_name);
    let mut total_documents = 0;
    let mut total_size_bytes = 0;
    let mut collections_stats = Vec::new();

    let collection_names = database
        .list_collection_names()
        .await
        .map_err(|err| format!("Can not get tables in database bcs: {}", err))?;

    for name in &collection_names {
        let collection = database.collection::<mongodb::bson::Document>(name);

        let mut cursor = match collection.find(doc! {}).await {
            Ok(c) => c,
            Err(err) => {
                eprintln!(
                    "Error fetching documents from collection '{}': {:?}",
                    name, err
                );
                continue;
            }
        };

        let mut document_count = 0;
        let mut collection_size = 0;

        while let Some(doc) = cursor.try_next().await.unwrap_or_else(|err| {
            eprintln!(
                "Error reading document from collection '{}': {:?}",
                name, err
            );
            None
        }) {
            document_count += 1;
            let doc_json = match to_string(&doc) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!(
                        "Error serializing document from collection '{}': {:?}",
                        name, err
                    );
                    continue;
                }
            };
            collection_size += doc_json.len();
        }

        // Aggregate results
        total_documents += document_count;
        total_size_bytes += collection_size;

        collections_stats.push(CollectionStats {
            name: name.clone(),
            document_count,
            size_bytes: format_bytes(collection_size),
        });
    }

    Ok(DatabaseStats {
        total_documents,
        total_collection: collection_names.len(),
        total_size_bytes: format_bytes(total_size_bytes),
        collections: collections_stats,
    })
}
