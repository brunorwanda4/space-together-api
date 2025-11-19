use crate::errors::AppError;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use serde::de::DeserializeOwned;

pub struct BaseRepository {
    pub collection: Collection<Document>,
}

impl BaseRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        Self { collection }
    }

    /// Generic fetch-all function with filtering, pagination, and deserialization
    pub async fn get_all<T: DeserializeOwned>(
        &self,
        filter: Option<String>,
        searchable_fields: &[&str],
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<(Vec<T>, i64, i64, i64), AppError> {
        let mut pipeline = vec![];

        // ===== BUILD MATCH STAGE =====
        let mut match_stage = if let Some(f) = filter.clone() {
            let regex = doc! {
                "$regex": f,
                "$options": "i"
            };

            let or_conditions: Vec<Document> = searchable_fields
                .iter()
                .map(|field| doc! { *field: &regex })
                .collect();

            doc! { "$or": or_conditions }
        } else {
            doc! {}
        };

        // ===== MERGE EXTRA MATCH =====
        if let Some(extra) = extra_match {
            if !extra.is_empty() {
                if !match_stage.is_empty() {
                    match_stage = doc! { "$and": [match_stage, extra] };
                } else {
                    match_stage = extra;
                }
            }
        }

        // Add match to pipeline if not empty
        if !match_stage.is_empty() {
            pipeline.push(doc! { "$match": &match_stage });
        }

        // ===== COUNT TOTAL DOCUMENTS =====
        let mut count_pipeline = vec![];
        if !match_stage.is_empty() {
            count_pipeline.push(doc! { "$match": &match_stage });
        }
        count_pipeline.push(doc! { "$count": "total" });

        let total_cursor = self
            .collection
            .aggregate(count_pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count documents: {}", e),
            })?;

        let results: Vec<Document> = total_cursor.try_collect().await.unwrap_or_default();

        let total = results
            .first()
            .and_then(|doc| {
                doc.get_i32("total")
                    .ok()
                    .map(|v| v as i64)
                    .or_else(|| doc.get_i64("total").ok())
            })
            .unwrap_or(0);

        // ===== PAGINATION SETUP =====
        let limit_value = limit.unwrap_or(50).max(1); // Avoid 0 or negative limit
        let skip_value = skip.unwrap_or(0);

        pipeline.push(doc! { "$sort": { "updated_at": -1 } });
        if skip_value > 0 {
            pipeline.push(doc! { "$skip": skip_value });
        }
        pipeline.push(doc! { "$limit": limit_value });

        // ===== FETCH DOCUMENTS =====
        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch documents: {}", e),
            })?;

        let mut items = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate documents: {}", e),
        })? {
            let item: T = mongodb::bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize document: {}", e),
            })?;
            items.push(item);
        }

        // ===== COMPUTE PAGINATION INFO =====
        let current_page = skip_value / limit_value + 1;
        let total_pages = if total > 0 {
            ((total as f64) / (limit_value as f64)).ceil() as i64
        } else {
            1
        };

        // Optional: Debug log
        // println!(
        //     "DEBUG -> total: {}, limit_value: {}, total_pages: {}, current_page: {}",
        //     total, limit_value, total_pages, current_page
        // );

        Ok((items, total, total_pages, current_page))
    }
}
