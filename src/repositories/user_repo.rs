use crate::errors::AppError;
use crate::services::user_service::normalize_user_ids;
use crate::{domain::user::User, models::id_model::IdType};
use futures::TryStreamExt;
use mongodb::bson;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct UserRepo {
    pub collection: Collection<User>,
}

impl UserRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<User>("users"),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let filter = doc! { "email": email };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find user by email: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let filter = doc! { "username": username };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find user by username: {}", e),
            })
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<User>, AppError> {
        let user_obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to format id: {}", e),
        })?;

        let filter = doc! { "_id": user_obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find user by id: {}", e),
            })
    }

    pub async fn insert_user(&self, user: &User) -> Result<User, AppError> {
        let index = IndexModel::builder()
            .keys(doc! {
                "email": 1,
                "username": 1
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create index: {}", e),
            })?;

        // Normalize IDs before inserting
        let mut user_to_insert = normalize_user_ids(user.clone())?;
        user_to_insert.id = None; // let MongoDB generate _id

        let res = self
            .collection
            .insert_one(&user_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert user: {}", e),
            })?;

        let inserted_object_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: format!(
                    "Failed to convert inserted_id ({:?}) to ObjectId",
                    res.inserted_id
                ),
            })?
            .to_owned();

        match self
            .find_by_id(&IdType::from_object_id(inserted_object_id))
            .await
        {
            Ok(Some(u)) => Ok(u),
            Ok(None) => Err(AppError {
                message: "User not found".to_string(),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        // Use aggregation to sort by updated_at descending
        let pipeline = vec![doc! { "$sort": { "updated_at": -1 } }];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch users: {}", e),
            })?;

        let mut users = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate users: {}", e),
        })? {
            // Deserialize into User
            let user: User = mongodb::bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize user: {}", e),
            })?;
            users.push(user);
        }

        Ok(users)
    }

    pub async fn update_user(&self, id: &IdType, updated_user: &User) -> Result<User, AppError> {
        let user_obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(updated_user).map_err(|e| AppError {
            message: format!("Failed to convert user to document: {}", e),
        })?;

        // Remove immutable field
        update_doc.remove("_id");

        let update_doc = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": user_obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update user: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "User not found after update".to_string(),
        })
    }

    pub async fn delete_user(&self, id: &IdType) -> Result<(), AppError> {
        let user_obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": user_obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete user: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No user deleted; user may not exist".to_string(),
            });
        }

        Ok(())
    }
}
