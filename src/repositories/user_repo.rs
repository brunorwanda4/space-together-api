use crate::errors::AppError;
use crate::utils::json_utils::change_insertoneresult_into_object_id;
// <- import your AppError
use crate::{domain::user::User, models::id_model::IdType};
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

        let res = self
            .collection
            .insert_one(user)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert user: {}", e),
            })?;

        match self
            .find_by_id(&IdType::from_object_id(
                change_insertoneresult_into_object_id(res),
            ))
            .await
        {
            Ok(Some(u)) => Ok(u),
            Ok(None) => Err(AppError {
                message: "User not found".to_string(),
            }),
            Err(e) => Err(e),
        }
    }
}
