use crate::{
    domain::message::Message,
    errors::AppError,
    models::id_model::IdType,
    helpers::object_id_helpers::parse_object_id,
};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOptions, IndexOptions},
    Collection, Database, IndexModel,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MessageService {
    collection: Collection<Message>,
    recent_message_ids: Arc<RwLock<HashSet<String>>>,
}

impl MessageService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("messages"),
            recent_message_ids: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexModel::builder()
                .keys(doc! { "conversation_id": 1, "created_at": -1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "school_id": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "sender.sender_id": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "client_message_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "deleted_at": 1 })
                .build(),
        ];

        self.collection
            .create_indexes(indexes)
            .await
            .map_err(|e| AppError { message: format!("Failed to create indexes: {}", e) })?;

        Ok(())
    }

    pub async fn create(&self, mut dto: Message) -> Result<Message, AppError> {
        {
            let mut ids = self.recent_message_ids.write().await;
            if ids.contains(&dto.client_message_id) {
                return Err(AppError { message: "Duplicate message detected".to_string() });
            }
            ids.insert(dto.client_message_id.clone());

            if ids.len() > 10000 {
                ids.clear();
            }
        }

        if dto.encrypted_payload.len() > 100 * 1024 {
            return Err(AppError { message: "Encrypted payload too large".to_string() });
        }

        dto.created_at = Utc::now();

        let result = self
            .collection
            .insert_one(&dto)
            .await
            .map_err(|e| AppError { message: format!("Failed to create message: {}", e) })?;

        dto.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(dto)
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Message, AppError> {
        let oid = parse_object_id(id).map_err(|e| AppError { message: e })?;

        self.collection
            .find_one(doc! { "_id": oid, "deleted_at": { "$exists": false } })
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?
            .ok_or_else(|| AppError { message: "Message not found".to_string() })
    }

    pub async fn get_conversation_messages(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Message>, i64), AppError> {
        let skip = (page - 1) * limit;

        let filter = doc! {
            "conversation_id": conversation_id,
            "deleted_at": { "$exists": false }
        };

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(limit)
            .build();

        let mut cursor = self
            .collection
            .find(filter.clone())
            .with_options(options)
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?;

        let mut messages = Vec::new();
        while cursor.advance().await.map_err(|e| AppError { message: format!("Database error: {}", e) })? {
            messages.push(cursor.deserialize_current().map_err(|e| AppError { message: format!("Deserialization error: {}", e) })?);
        }

        let total = self
            .collection
            .count_documents(filter)
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?;

        Ok((messages, total as i64))
    }

    pub async fn get_conversation_files(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Message>, i64), AppError> {
        let skip = (page - 1) * limit;

        let filter = doc! {
            "conversation_id": conversation_id,
            "message_type": "FILE",
            "deleted_at": { "$exists": false }
        };

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(limit)
            .build();

        let mut cursor = self
            .collection
            .find(filter.clone())
            .with_options(options)
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?;

        let mut messages = Vec::new();
        while cursor.advance().await.map_err(|e| AppError { message: format!("Database error: {}", e) })? {
            messages.push(cursor.deserialize_current().map_err(|e| AppError { message: format!("Deserialization error: {}", e) })?);
        }

        let total = self
            .collection
            .count_documents(filter)
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?;

        Ok((messages, total as i64))
    }

    pub async fn soft_delete(&self, id: &IdType) -> Result<Message, AppError> {
        let oid = parse_object_id(id).map_err(|e| AppError { message: e })?;

        self.collection
            .find_one_and_update(
                doc! { "_id": oid },
                doc! { "$set": { "deleted_at": mongodb::bson::to_bson(&Utc::now()).unwrap() } },
            )
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?
            .ok_or_else(|| AppError { message: "Message not found".to_string() })
    }
}
