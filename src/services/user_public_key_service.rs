use crate::{
    domain::user_public_key::{PublicKeyInfo, UserPublicKey},
    errors::AppError,
    models::mongo_model::IndexDef,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

pub struct UserPublicKeyService {
    pub collection: Collection<UserPublicKey>,
}

impl UserPublicKeyService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<UserPublicKey>("user_public_keys"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![IndexDef::single("user_id", true)];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;

        Ok(())
    }

    // =========================
    // UPSERT PUBLIC KEY
    // =========================
    pub async fn upsert_public_key(
        &self,
        user_id: ObjectId,
        public_key: String,
        key_algorithm: String,
    ) -> Result<UserPublicKey, AppError> {
        self.ensure_indexes().await?;

        // Validate PEM format
        if !public_key.contains("BEGIN PUBLIC KEY") || !public_key.contains("END PUBLIC KEY") {
            return Err(AppError {
                message: "Invalid public key format. Must be PEM format.".to_string(),
            });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Check if key already exists
        let existing = repo
            .find_one::<UserPublicKey>(doc! { "user_id": user_id }, None)
            .await?;

        if let Some(existing_key) = existing {
            // Update existing key
            let update_doc = doc! {
                "$set": {
                    "public_key": &public_key,
                    "key_algorithm": &key_algorithm,
                    "updated_at": mongodb::bson::to_bson(&chrono::Utc::now()).unwrap()
                }
            };

            repo.update_one_raw(&crate::models::id_model::IdType::ObjectId(existing_key.id.unwrap()), update_doc)
                .await?;

            self.get_public_key(user_id).await
        } else {
            // Create new key
            let new_key = UserPublicKey {
                id: None,
                user_id,
                public_key,
                key_algorithm,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let doc = mongodb::bson::to_document(&new_key).map_err(|e| AppError {
                message: format!("Failed to serialize public key: {}", e),
            })?;

            repo.create::<UserPublicKey>(extract_valid_fields(doc), None)
                .await
        }
    }

    // =========================
    // GET PUBLIC KEY
    // =========================
    pub async fn get_public_key(&self, user_id: ObjectId) -> Result<UserPublicKey, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<UserPublicKey>(doc! { "user_id": user_id }, None)
            .await?
            .ok_or(AppError {
                message: format!("Public key not found for user: {}", user_id),
            })
    }

    // =========================
    // GET MULTIPLE PUBLIC KEYS
    // =========================
    pub async fn get_public_keys(
        &self,
        user_ids: Vec<ObjectId>,
    ) -> Result<Vec<PublicKeyInfo>, AppError> {
        if user_ids.is_empty() {
            return Err(AppError {
                message: "user_ids parameter is required".to_string(),
            });
        }

        if user_ids.len() > 50 {
            return Err(AppError {
                message: "Maximum 50 user IDs allowed per request".to_string(),
            });
        }

        let filter = doc! { "user_id": { "$in": user_ids.clone() } };

        let cursor = self
            .collection
            .find(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch public keys: {}", e),
            })?;

        use futures::stream::TryStreamExt;
        let keys: Vec<UserPublicKey> = cursor
            .try_collect()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to collect public keys: {}", e),
            })?;

        // Check if all requested users have public keys
        if keys.len() != user_ids.len() {
            let found_ids: Vec<ObjectId> = keys.iter().map(|k| k.user_id).collect();
            let missing_ids: Vec<String> = user_ids
                .iter()
                .filter(|id| !found_ids.contains(id))
                .map(|id| id.to_hex())
                .collect();

            return Err(AppError {
                message: format!(
                    "Public key not found for users: {}",
                    missing_ids.join(", ")
                ),
            });
        }

        // Convert to PublicKeyInfo
        let public_keys: Vec<PublicKeyInfo> = keys
            .into_iter()
            .map(|k| PublicKeyInfo {
                user_id: k.user_id.to_hex(),
                public_key: k.public_key,
                key_algorithm: k.key_algorithm,
                created_at: k.created_at,
            })
            .collect();

        Ok(public_keys)
    }

    // =========================
    // DELETE PUBLIC KEY
    // =========================
    pub async fn delete_public_key(&self, user_id: ObjectId) -> Result<(), AppError> {
        self.collection
            .delete_one(doc! { "user_id": user_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete public key: {}", e),
            })?;

        Ok(())
    }
}
