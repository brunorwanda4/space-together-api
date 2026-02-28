use crate::{
    domain::message::{Message, MessageWithRelations},
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::IndexDef,
    },
    pipeline::message_pipeline::message_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MessageService {
    pub collection: Collection<Message>,
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
            IndexDef::compound(vec![("conversation_id", 1), ("created_at", -1)], false),
            IndexDef::single("school_id", false),
            IndexDef::single("sender.id", false),
            IndexDef::single("client_message_id", true),
            IndexDef::single("deleted_at", false),
        ];

        let repo = BaseRepository::new(
            self.collection
                .clone()
                .clone_with_type::<Document>(),
        );

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(&self, dto: Message) -> Result<Message, AppError> {
        self.ensure_indexes().await?;

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

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let message = repo
            .create::<Message>(extract_valid_fields(dto.to_document()?), Some(&["client_message_id"]))
            .await?;

        Ok(message)
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Message, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! {
            "_id": IdType::to_object_id(id)?,
            "deleted_at": { "$exists": false }
        };

        repo.find_one::<Message>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Message not found".into(),
            })
    }

    pub async fn get_conversation_messages(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Message>, i64), AppError> {
        let skip = (page - 1) * limit;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let extra_match = doc! {
            "conversation_id": conversation_id,
            "deleted_at": { "$exists": false }
        };

        let (messages, total, _, _) = repo
            .get_all::<Message>(None, &[], Some(limit), Some(skip), Some(extra_match))
            .await?;

        Ok((messages, total))
    }

    pub async fn get_conversation_files(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Message>, i64), AppError> {
        let skip = (page - 1) * limit;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let extra_match = doc! {
            "conversation_id": conversation_id,
            "message_type": "FILE",
            "deleted_at": { "$exists": false }
        };

        let (messages, total, _, _) = repo
            .get_all::<Message>(None, &[], Some(limit), Some(skip), Some(extra_match))
            .await?;

        Ok((messages, total))
    }

    pub async fn soft_delete(&self, id: &IdType) -> Result<Message, AppError> {
        let message = self.find_one(id).await?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let update_doc = doc! {
            "$set": {
                "deleted_at": mongodb::bson::to_bson(&Utc::now()).unwrap()
            }
        };

        repo.update_one_raw(id, update_doc).await?;

        Ok(message)
    }

    // =========================
    // GET CONVERSATION MESSAGES WITH RELATIONS
    // =========================
    pub async fn get_conversation_messages_with_relations(
        &self,
        conversation_id: ObjectId,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<MessageWithRelations>, i64), AppError> {
        let skip = (page - 1) * limit;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let match_stage = doc! {
            "conversation_id": conversation_id,
            "deleted_at": { "$exists": false }
        };

        let result = repo
            .aggregate_with_paginate::<MessageWithRelations>(
                message_pipeline(match_stage),
                Some(limit),
                Some(skip),
            )
            .await?;

        Ok((result.data, result.total))
    }

    // =========================
    // FIND ONE WITH RELATIONS
    // =========================
    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
    ) -> Result<MessageWithRelations, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let match_stage = doc! {
            "_id": IdType::to_object_id(id)?,
            "deleted_at": { "$exists": false }
        };

        repo.aggregate_one::<MessageWithRelations>(message_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Message not found".into(),
            })
    }
}
