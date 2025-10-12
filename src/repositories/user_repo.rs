use crate::domain::user::UserStats;
use crate::errors::AppError;
use crate::services::user_service::normalize_user_ids;
use crate::{domain::user::User, models::id_model::IdType};
use chrono::{Duration, Utc};
use futures::TryStreamExt;
use mongodb::{bson, IndexModel};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    options::IndexOptions,
    Collection, Database,
};
use std::time::SystemTime;

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

    pub async fn get_all_users(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<User>, AppError> {
        let mut pipeline = vec![];

        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"
            };
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "email": &regex },
                        { "role": &regex },
                        { "username": &regex },
                        { "phone": &regex },
                        { "gender": &regex },
                        { "age": &regex },
                        { "address": &regex },
                        { "bio": &regex },
                    ]
                }
            });
        }

        pipeline.push(doc! {
            "$addFields": {
                "sort_date": { "$ifNull": [ "$updated_at", "$created_at" ] }
            }
        });
        pipeline.push(doc! { "$sort": { "updated_at": -1 } });

        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }
        pipeline.push(doc! { "$limit": limit.unwrap_or(10) });

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
        update_doc.remove("_id");

        self.collection
            .update_one(doc! { "_id": user_obj_id }, doc! { "$set": update_doc })
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

    pub async fn get_user_stats(&self) -> Result<UserStats, AppError> {
        let total = self.collection.count_documents(doc! {}).await?;
        let male = self
            .collection
            .count_documents(doc! { "gender": "MALE" })
            .await?;
        let female = self
            .collection
            .count_documents(doc! { "gender": "FEMALE" })
            .await?;
        let other = self
            .collection
            .count_documents(doc! { "gender": "OTHER" })
            .await?;
        let admins = self
            .collection
            .count_documents(doc! { "role": "ADMIN" })
            .await?;
        let staff = self
            .collection
            .count_documents(doc! { "role": "SCHOOLSTAFF" })
            .await?;
        let students = self
            .collection
            .count_documents(doc! { "role": "STUDENT" })
            .await?;
        let teachers = self
            .collection
            .count_documents(doc! { "role": "TEACHER" })
            .await?;
        let assigned_school = self
            .collection
            .count_documents(doc! { "current_school_id": { "$exists": true } })
            .await?;
        let no_school = self
            .collection
            .count_documents(doc! { "current_school_id": { "$exists": false } })
            .await?;

        // 🔹 Fix for recent 30 days
        let thirty_days_ago_chrono = Utc::now() - Duration::days(30);
        let thirty_days_ago_system: SystemTime = thirty_days_ago_chrono.into();
        let thirty_days_ago_bson = BsonDateTime::from_system_time(thirty_days_ago_system);

        let recent_30_days_count = self
            .collection
            .count_documents(doc! { "created_at": { "$gte": thirty_days_ago_bson } })
            .await?;

        Ok(UserStats {
            total: total as i64,
            male: male as i64,
            female: female as i64,
            other: other as i64,
            admins: admins as i64,
            staff: staff as i64,
            students: students as i64,
            teachers: teachers as i64,
            assigned_school: assigned_school as i64,
            no_school: no_school as i64,
            recent_30_days: recent_30_days_count as i64,
        })
    }

    pub async fn add_school_to_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        let user_obj_id = ObjectId::parse_str(user_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid user id: {}", e),
        })?;
        let school_obj_id = ObjectId::parse_str(school_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid school id: {}", e),
        })?;

        let filter = doc! { "_id": user_obj_id };
        let update = doc! {
            "$addToSet": { "schools": school_obj_id }, // avoid duplicates
            "$set": {
             "current_school_id": school_obj_id,
             "updated_at": Utc::now().to_rfc3339(),
          }
        };

        self.collection
            .update_one(filter.clone(), update)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to add school to user: {}", e),
            })?;

        // Return updated user
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated user: {}", e),
            })?
            .ok_or_else(|| AppError {
                message: "User not found after update".to_string(),
            })
    }

    /// Remove a school ID from a user's schools array
    pub async fn remove_school_from_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        let user_obj_id = ObjectId::parse_str(user_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid user id: {}", e),
        })?;
        let school_obj_id = ObjectId::parse_str(school_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid school id: {}", e),
        })?;

        let filter = doc! { "_id": user_obj_id };

        let update = doc! {
            "$pull": { "schools": school_obj_id },
           "$set": { "updated_at": Utc::now().to_rfc3339() }
        };

        self.collection
            .update_one(filter.clone(), update)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to remove school from user: {}", e),
            })?;

        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated user: {}", e),
            })?
            .ok_or_else(|| AppError {
                message: "User not found after update".to_string(),
            })
    }
}
