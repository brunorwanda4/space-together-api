use crate::{
    domain::{
        common_details::Paginated,
        conversation::{Conversation, ConversationKey},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::IndexDef,
    },
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

pub struct ConversationService {
    pub collection: Collection<Conversation>,
    pub keys_collection: Collection<ConversationKey>,
}

impl ConversationService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Conversation>("conversations"),
            keys_collection: db.collection::<ConversationKey>("conversation_keys"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("school_id", false),
            IndexDef::single("participants.user.id", false),
            IndexDef::single("created_at", false),
        ];

        let repo = BaseRepository::new(
            self.collection.clone().clone_with_type::<Document>(),
        );
        repo.ensure_indexes(&indexes).await?;

        let key_indexes = vec![
            IndexDef::compound(vec![("conversation_id", 1), ("user_id", 1)], true),
        ];

        let key_repo = BaseRepository::new(
            self.keys_collection.clone().clone_with_type::<Document>(),
        );
        key_repo.ensure_indexes(&key_indexes).await?;

        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: Conversation) -> Result<Conversation, AppError> {
        self.ensure_indexes().await?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let doc = mongodb::bson::to_document(&dto).map_err(|e| AppError {
            message: format!("Failed to serialize conversation: {}", e),
        })?;

        repo.create::<Conversation>(extract_valid_fields(doc), None)
            .await
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Conversation, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Conversation>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Conversation not found".into(),
            })
    }

    // =========================
    // GET ALL
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Conversation>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "_id",
            "school_id",
            "participants.user.id",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Conversation>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // =========================
    // CONVERSATION KEYS
    // =========================
    pub async fn store_conversation_key(&self, key: ConversationKey) -> Result<ConversationKey, AppError> {
        let repo = BaseRepository::new(
            self.keys_collection.clone().clone_with_type::<Document>(),
        );

        let doc = mongodb::bson::to_document(&key).map_err(|e| AppError {
            message: format!("Failed to serialize conversation key: {}", e),
        })?;

        repo.create::<ConversationKey>(
            extract_valid_fields(doc),
            None,
        )
        .await
    }

    pub async fn get_conversation_key(
        &self,
        conversation_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<ConversationKey, AppError> {
        let repo = BaseRepository::new(
            self.keys_collection.clone().clone_with_type::<Document>(),
        );

        repo.find_one::<ConversationKey>(
            doc! { "conversation_id": conversation_id, "user_id": user_id },
            None,
        )
        .await?
        .ok_or(AppError {
            message: "Conversation key not found".into(),
        })
    }

    // =========================
    // PARTICIPANT CHECK
    // =========================
    pub async fn is_participant(
        &self,
        conversation_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<bool, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let result = repo
            .find_one::<Conversation>(
                doc! {
                    "_id": conversation_id,
                    "participants.user.id": user_id
                },
                None,
            )
            .await?;

        Ok(result.is_some())
    }
}
