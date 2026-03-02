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
        // Ensure indexes exist
        self.ensure_indexes().await?;
        
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<UserPublicKey>(doc! { "user_id": user_id.to_hex() }, None)
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
    // GET MULTIPLE PUBLIC KEYS (PARTIAL)
    // Returns found keys and list of missing user IDs
    // =========================
    pub async fn get_public_keys_partial(
        &self,
        user_ids: Vec<ObjectId>,
    ) -> Result<(Vec<PublicKeyInfo>, Vec<String>), AppError> {
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

        // Ensure indexes exist
        self.ensure_indexes().await?;

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

        // Identify missing user IDs
        let found_ids: Vec<ObjectId> = keys.iter().map(|k| k.user_id).collect();
        let missing_ids: Vec<String> = user_ids
            .iter()
            .filter(|id| !found_ids.contains(id))
            .map(|id| id.to_hex())
            .collect();

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

        Ok((public_keys, missing_ids))
    }

    // =========================
    // GET OR CREATE PUBLIC KEYS
    // Automatically generates keys for users who don't have them
    // =========================
    pub async fn get_or_create_public_keys(
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

        // First, try to get existing keys
        let (mut public_keys, missing_ids) = self.get_public_keys_partial(user_ids.clone()).await?;

        // Generate keys for missing users
        if !missing_ids.is_empty() {
            for missing_id_str in missing_ids {
                let user_id = ObjectId::parse_str(&missing_id_str).map_err(|e| AppError {
                    message: format!("Failed to parse user ID: {}", e),
                })?;

                // Try to create key, but handle race condition where key was created by another request
                match self.try_create_public_key(user_id).await {
                    Ok(created_key) => {
                        // Successfully created new key
                        public_keys.push(PublicKeyInfo {
                            user_id: created_key.user_id.to_hex(),
                            public_key: created_key.public_key,
                            key_algorithm: created_key.key_algorithm,
                            created_at: created_key.created_at,
                        });
                    }
                    Err(e) if e.message.contains("duplicate key error") || e.message.contains("E11000") => {
                        // Key was created by another request, fetch it
                        match self.get_public_key(user_id).await {
                            Ok(existing_key) => {
                                public_keys.push(PublicKeyInfo {
                                    user_id: existing_key.user_id.to_hex(),
                                    public_key: existing_key.public_key,
                                    key_algorithm: existing_key.key_algorithm,
                                    created_at: existing_key.created_at,
                                });
                            }
                            Err(fetch_err) => {
                                // If we still can't fetch it, return the original error
                                return Err(AppError {
                                    message: format!(
                                        "Failed to create or fetch public key for user {}: {}",
                                        user_id, fetch_err.message
                                    ),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Other error, propagate it
                        return Err(e);
                    }
                }
            }
        }

        Ok(public_keys)
    }

    // =========================
    // TRY CREATE PUBLIC KEY
    // Attempts to create a new public key without checking if it exists first
    // =========================
    async fn try_create_public_key(&self, user_id: ObjectId) -> Result<UserPublicKey, AppError> {
        // Ensure indexes exist (including unique index on user_id)
        self.ensure_indexes().await?;
        
        // Generate a new public key
        let public_key = crate::utils::crypto_utils::generate_rsa_public_key()?;
        
        // Try to insert directly (will fail if key already exists due to unique index)
        let new_key = UserPublicKey {
            id: None,
            user_id,
            public_key,
            key_algorithm: "RSA-2048".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let doc = mongodb::bson::to_document(&new_key).map_err(|e| AppError {
            message: format!("Failed to serialize public key: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<UserPublicKey>(extract_valid_fields(doc), None)
            .await
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
