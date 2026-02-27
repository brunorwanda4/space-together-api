use crate::{
    domain::conversation::{Conversation, ConversationKey},
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

pub struct ConversationService {
    collection: Collection<Conversation>,
    keys_collection: Collection<ConversationKey>,
}

impl ConversationService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("conversations"),
            keys_collection: db.collection("conversation_keys"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexModel::builder()
                .keys(doc! { "school_id": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "participants.user.id": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "created_at": -1 })
                .build(),
        ];

        self.collection
            .create_indexes(indexes)
            .await
            .map_err(|e| AppError { message: format!("Failed to create indexes: {}", e) })?;

        let key_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "conversation_id": 1, "user_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ];

        self.keys_collection
            .create_indexes(key_indexes)
            .await
            .map_err(|e| AppError { message: format!("Failed to create key indexes: {}", e) })?;

        Ok(())
    }

    pub async fn create(&self, mut dto: Conversation) -> Result<Conversation, AppError> {
        dto.created_at = Utc::now();
        dto.updated_at = Utc::now();

        let result = self
            .collection
            .insert_one(&dto)
            .await
            .map_err(|e| AppError { message: format!("Failed to create conversation: {}", e) })?;

        dto.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(dto)
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Conversation, AppError> {
        let oid = parse_object_id(id).map_err(|e| AppError { message: e })?;

        self.collection
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?
            .ok_or_else(|| AppError { message: "Conversation not found".to_string() })
    }

    pub async fn get_all(
        &self,
        filter: mongodb::bson::Document,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Conversation>, i64), AppError> {
        let skip = (page - 1) * limit;

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

        let mut conversations = Vec::new();
        while cursor.advance().await.map_err(|e| AppError { message: format!("Database error: {}", e) })? {
            conversations.push(cursor.deserialize_current().map_err(|e| AppError { message: format!("Deserialization error: {}", e) })?);
        }

        let total = self
            .collection
            .count_documents(filter)
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?;

        Ok((conversations, total as i64))
    }

    pub async fn store_conversation_key(&self, key: ConversationKey) -> Result<(), AppError> {
        self.keys_collection
            .insert_one(&key)
            .await
            .map_err(|e| AppError { message: format!("Failed to store conversation key: {}", e) })?;

        Ok(())
    }

    pub async fn get_conversation_key(
        &self,
        conversation_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<ConversationKey, AppError> {
        self.keys_collection
            .find_one(doc! { "conversation_id": conversation_id, "user_id": user_id })
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?
            .ok_or_else(|| AppError { message: "Conversation key not found".to_string() })
    }

    pub async fn is_participant(
        &self,
        conversation_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<bool, AppError> {
        let count = self
            .collection
            .count_documents(doc! {
                "_id": conversation_id,
                "participants.user.id": user_id
            })
            .await
            .map_err(|e| AppError { message: format!("Database error: {}", e) })?;

        Ok(count > 0)
    }
}
