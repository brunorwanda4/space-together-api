use crate::domain::user::{UpdateUserDto, User, UserStats};
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::services::user_service::normalize_user_ids;
use chrono::{Duration, Utc, Weekday};
use futures::TryStreamExt;
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

    // =========================================================
    // 🔹 Basic CRUD Operations
    // =========================================================

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

    // =========================================================
    // 🔹 Insert New User
    // =========================================================
    pub async fn insert_user(&self, user: &mut User) -> Result<User, AppError> {
        // Ensure unique email & username
        let index_email = IndexModel::builder()
            .keys(doc! { "email": 1,})
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let index_username = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_index(index_email)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create index email: {}", e),
            })?;

        self.collection
            .create_index(index_username)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create index username: {}", e),
            })?;

        let mut user_to_insert = normalize_user_ids(user.clone())?;
        user_to_insert.id = None;

        // ✅ Ensure timestamps and default availability
        user_to_insert.created_at = Some(Utc::now());
        user_to_insert.updated_at = Some(Utc::now());

        if user_to_insert.availability_schedule.is_none() {
            use crate::domain::common_details::{DailyAvailability, TimeRange};

            let default_availability = vec![
                DailyAvailability {
                    day: Weekday::Mon,
                    time_range: TimeRange::new("09:00", "17:00"),
                },
                DailyAvailability {
                    day: Weekday::Tue,
                    time_range: TimeRange::new("09:00", "17:00"),
                },
                DailyAvailability {
                    day: Weekday::Wed,
                    time_range: TimeRange::new("09:00", "17:00"),
                },
                DailyAvailability {
                    day: Weekday::Thu,
                    time_range: TimeRange::new("09:00", "17:00"),
                },
                DailyAvailability {
                    day: Weekday::Fri,
                    time_range: TimeRange::new("09:00", "17:00"),
                },
            ];

            user_to_insert.availability_schedule = Some(default_availability);
        }

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

        self.find_by_id(&IdType::from_object_id(inserted_object_id))
            .await?
            .ok_or(AppError {
                message: "User not found after insert".to_string(),
            })
    }

    // =========================================================
    // 🔹 Fetch All Users (with optional search & pagination)
    // =========================================================
    pub async fn get_all_users(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<User>, AppError> {
        let mut pipeline = vec![];

        if let Some(f) = filter {
            let regex = doc! { "$regex": f.clone(), "$options": "i" };
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "email": &regex },
                        { "username": &regex },
                        { "phone": &regex },
                        { "role": &regex },
                        { "gender": &regex },
                        { "bio": &regex },
                        { "dream_career": &regex },
                        { "languages_spoken": &regex },
                        { "hobbies_interests": &regex },
                        { "special_skills": &regex },
                        { "department": &regex },
                        { "job_title": &regex },
                        { "preferred_communication_method": &regex }
                    ]
                }
            });
        }

        pipeline.push(doc! {
            "$addFields": {
                "sort_date": { "$ifNull": [ "$updated_at", "$created_at" ] }
            }
        });
        pipeline.push(doc! { "$sort": { "sort_date": -1 } });

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
            let user: User = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize user: {}", e),
            })?;
            users.push(user);
        }

        Ok(users)
    }

    // =========================================================
    // 🔹 Update User (Partial Update via UpdateUserDto)
    // =========================================================
    pub async fn update_user_fields(
        &self,
        id: &str,
        update_dto: &UpdateUserDto,
    ) -> Result<User, AppError> {
        let user_obj_id = ObjectId::parse_str(id).map_err(|e| AppError {
            message: format!("Invalid id: {}", e),
        })?;

        // Convert DTO → BSON document
        let mut update_doc = bson::to_document(update_dto).map_err(|e| AppError {
            message: format!("Failed to convert update dto to document: {}", e),
        })?;

        // Filter out null values
        update_doc = update_doc
            .into_iter()
            .filter(|(_, v)| !matches!(v, bson::Bson::Null))
            .collect();

        if update_doc.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".to_string(),
            });
        }

        // Always update `updated_at`
        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        // Perform update
        self.collection
            .update_one(doc! { "_id": user_obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update user: {}", e),
            })?;

        // Fetch updated user
        let updated_user = self
            .collection
            .find_one(doc! { "_id": user_obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Error fetching updated user: {}", e),
            })?;

        updated_user.ok_or(AppError {
            message: "User not found after update".to_string(),
        })
    }

    // =========================================================
    // 🔹 Delete User
    // =========================================================
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

    // =========================================================
    // 🔹 Statistics
    // =========================================================
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

    // =========================================================
    // 🔹 Add & Remove Schools
    // =========================================================
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

        let filter = doc! { "_id": &user_obj_id };

        self.collection
            .update_one(
                filter.clone(),
                doc! {
                    "$addToSet": { "schools": &school_obj_id },
                    "$set": {
                        "current_school_id": &school_obj_id,
                        "updated_at": bson::to_bson(&Utc::now()).unwrap()
                    }
                },
            )
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
            .ok_or(AppError {
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

        let filter = doc! { "_id": &user_obj_id };
        let update = doc! {
            "$pull": { "schools": &school_obj_id },
            "$set": { "updated_at": bson::to_bson(&Utc::now()).unwrap() }
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
            .ok_or(AppError {
                message: "User not found after update".to_string(),
            })
    }
}
