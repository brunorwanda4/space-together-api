use crate::errors::AppError;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, Bson, Document},
    Collection,
};
use serde::de::DeserializeOwned;

/// Convert a BSON document into a Rust struct via serde_json
fn bson_doc_to_struct<T>(doc: Document) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    // Convert BSON Document → serde_json::Value
    let json_value: serde_json::Value =
        bson::from_bson(Bson::Document(doc)).map_err(|e| AppError {
            message: format!("Failed to convert BSON to JSON: {}", e),
        })?;

    // Convert serde_json::Value → Rust struct
    serde_json::from_value(json_value).map_err(|e| AppError {
        message: format!("Failed to deserialize aggregate result: {}", e),
    })
}

/// Run an aggregation pipeline and return a single item.
pub async fn aggregate_single<T>(
    collection: &Collection<Document>,
    pipeline: Vec<Document>,
) -> Result<Option<T>, AppError>
where
    T: DeserializeOwned,
{
    let mut cursor = collection.aggregate(pipeline).await.map_err(|e| AppError {
        message: format!("Aggregation failed: {}", e),
    })?;

    if let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
        message: format!("Failed to iterate aggregate cursor: {}", e),
    })? {
        let item = bson_doc_to_struct(doc)?;
        Ok(Some(item))
    } else {
        Ok(None)
    }
}

/// Run an aggregation pipeline and return multiple items.
pub async fn aggregate_many<T>(
    collection: &Collection<Document>,
    pipeline: Vec<Document>,
) -> Result<Vec<T>, AppError>
where
    T: DeserializeOwned,
{
    let mut cursor = collection.aggregate(pipeline).await.map_err(|e| AppError {
        message: format!("Aggregation failed: {}", e),
    })?;

    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
        message: format!("Failed to iterate aggregate cursor: {}", e),
    })? {
        // ✅ This handles both single objects and arrays correctly
        let item = bson_doc_to_struct(doc)?;
        results.push(item);
    }

    Ok(results)
}
