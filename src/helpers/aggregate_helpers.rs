use futures::TryStreamExt;
use mongodb::{
    bson::{self, Document},
    Collection,
};

use crate::errors::AppError;

/// Run an aggregation pipeline and return a single item.
pub async fn aggregate_single<T>(
    collection: &Collection<Document>,
    pipeline: Vec<Document>,
) -> Result<Option<T>, AppError>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let mut cursor = collection.aggregate(pipeline).await.map_err(|e| AppError {
        message: format!("Aggregation failed: {}", e),
    })?;

    if let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
        message: format!("Failed to iterate aggregate cursor: {}", e),
    })? {
        let item = bson::from_document(doc).map_err(|e| AppError {
            message: format!("Failed to deserialize aggregate result: {}", e),
        })?;
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
    for<'de> T: serde::Deserialize<'de>,
{
    let mut cursor = collection.aggregate(pipeline).await.map_err(|e| AppError {
        message: format!("Aggregation failed: {}", e),
    })?;

    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
        message: format!("Failed to iterate aggregate cursor: {}", e),
    })? {
        results.push(bson::from_document(doc).map_err(|e| AppError {
            message: format!("Failed to deserialize aggregate result: {}", e),
        })?);
    }
    Ok(results)
}
