use crate::domain::user::{UpdateUserDto, User, UserStats};
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::services::user_service::normalize_user_ids;
use chrono::{Duration, Utc};
use futures::TryStreamExt;
use mongodb::bson::Document;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime as BsonDateTime},
    options::IndexOptions,
    Collection, Database, IndexModel,
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
        // Ensure unique email & username
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

        let mut user_to_insert = normalize_user_ids(user.clone())?;
        user_to_insert.id = None;

        let res = self
            .collection
            .insert_one(&user_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert user: {}", e),
            })?;

        let inserted_object_id = res.inserted_id.as_object_id().ok_or_else(|| AppError {
            message: "Failed to convert inserted_id to ObjectId".to_string(),
        })?;

        match self
            .find_by_id(&IdType::from_object_id(inserted_object_id))
            .await
        {
            Ok(Some(u)) => Ok(u),
            _ => Err(AppError {
                message: "User not found after insert".to_string(),
            }),
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
                        { "bio": &regex },
                        { "dream_career": &regex },
                        { "hobbies_interests": &regex },
                        { "special_skills": &regex },
                        { "languages_spoken": &regex },
                        { "preferred_communication_method": &regex },
                        { "learning_challenges": &regex },
                        { "special_support_needed": &regex },
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

    /// ✅ Updated version — partial update with `UpdateUserDto`
    pub async fn update_user_fields(
        &self,
        id: &str,
        update_dto: &UpdateUserDto,
    ) -> Result<User, AppError> {
        let user_obj_id = ObjectId::parse_str(id).map_err(|e| AppError {
            message: format!("Invalid id: {}", e),
        })?;

        // Convert UpdateUserDto to BSON Document
        let update_doc_full = bson::to_document(update_dto).map_err(|e| AppError {
            message: format!("Failed to convert update dto to document: {}", e),
        })?;

        // Filter out null or None fields
        let update_doc: Document = update_doc_full
            .into_iter()
            .filter(|(_, v)| !matches!(v, bson::Bson::Null))
            .collect();

        if update_doc.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".to_string(),
            });
        }

        // Perform the update
        self.collection
            .update_one(doc! { "_id": user_obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update user: {}", e),
            })?;

        // Return updated document
        let updated_user = self
            .collection
            .find_one(doc! { "_id": user_obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Error fetching updated user: {}", e),
            })?
            .ok_or(AppError {
                message: "User not found after update".to_string(),
            })?;

        Ok(updated_user)
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

        let thirty_days_ago = Utc::now() - Duration::days(30);
        let thirty_days_bson = BsonDateTime::from_system_time(SystemTime::from(thirty_days_ago));

        let recent_30_days = self
            .collection
            .count_documents(doc! { "created_at": { "$gte": thirty_days_bson } })
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
            recent_30_days: recent_30_days as i64,
        })
    }

    pub async fn add_school_to_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        use mongodb::bson::oid::ObjectId;

        let user_obj_id = ObjectId::parse_str(user_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid user id: {}", e),
        })?;
        let school_obj_id = ObjectId::parse_str(school_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid school id: {}", e),
        })?;

        let filter = doc! { "_id": &user_obj_id };

        let user_doc = self
            .collection
            .find_one(filter.clone())
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch user: {}", e),
            })?;

        if let Some(user_doc) = user_doc {
            let raw_doc = mongodb::bson::to_document(&user_doc).unwrap_or_default();
            let needs_init = match raw_doc.get("schools") {
                Some(mongodb::bson::Bson::Array(_)) => false,
                _ => true,
            };
            if needs_init {
                self.collection
                    .update_one(
                        doc! { "_id": &user_obj_id },
                        doc! { "$set": { "schools": bson::to_bson(&Vec::<ObjectId>::new()).unwrap() } },
                    )
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to init schools: {}", e),
                    })?;
            }
        } else {
            return Err(AppError {
                message: "User not found".to_string(),
            });
        }

        let update = doc! {
            "$addToSet": { "schools": &school_obj_id },
            "$set": {
                "current_school_id": &school_obj_id,
                "updated_at": Utc::now().to_rfc3339(),
            }
        };

        self.collection
            .update_one(filter.clone(), update)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to add school: {}", e),
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
                message: format!("Failed to remove school: {}", e),
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
