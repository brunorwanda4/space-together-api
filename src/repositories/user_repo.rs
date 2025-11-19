use crate::domain::user::{PaginatedUsers, UpdateUserDto, User, UserStats};
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::repositories::base_repo::BaseRepository;
use crate::services::user_service::normalize_user_ids;
use chrono::{Duration, Utc, Weekday};
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime as BsonDateTime, Document},
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
    // 🔹 Utility Helpers
    // =========================================================

    fn obj_id_from_str(id: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(id).map_err(|e| AppError {
            message: format!("Invalid ObjectId: {}", e),
        })
    }

    fn obj_id_from_idtype(id: &IdType) -> Result<ObjectId, AppError> {
        Self::obj_id_from_str(&id.as_string()) // ✅ borrow fix
    }

    async fn find_one_by_filter(&self, filter: Document) -> Result<Option<User>, AppError> {
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Database query failed: {}", e),
            })
    }

    async fn update_one_and_fetch(
        &self,
        filter: Document,
        update: Document,
    ) -> Result<User, AppError> {
        self.collection
            .update_one(filter.clone(), update)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update user: {}", e),
            })?;

        self.find_one_by_filter(filter).await?.ok_or(AppError {
            message: "User not found after update".to_string(),
        })
    }

    // =========================================================
    // 🔹 Find User
    // =========================================================

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        self.find_one_by_filter(doc! { "email": email }).await
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        self.find_one_by_filter(doc! { "username": username }).await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<User>, AppError> {
        let obj_id = Self::obj_id_from_idtype(id)?;
        self.find_one_by_filter(doc! { "_id": obj_id }).await
    }

    // =========================================================
    // 🔹 Insert User
    // =========================================================

    pub async fn insert_user(&self, user: &mut User) -> Result<User, AppError> {
        // Unique indexes
        for field in ["email", "username"] {
            let index = IndexModel::builder()
                .keys(doc! { field: 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build();
            self.collection
                .create_index(index)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to create unique index on {}: {}", field, e),
                })?;
        }

        // Normalize and set timestamps
        let mut user_to_insert = normalize_user_ids(user.clone())?;
        user_to_insert.id = None;
        user_to_insert.created_at = Some(Utc::now());
        user_to_insert.updated_at = Some(Utc::now());

        // Default availability (✅ fixed)
        if user_to_insert.availability_schedule.is_none() {
            use crate::domain::common_details::{DailyAvailability, TimeRange};

            let availability: Vec<DailyAvailability> = vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ]
            .into_iter()
            .map(|day| DailyAvailability {
                day,
                time_range: TimeRange::new("09:00", "17:00"),
            })
            .collect();

            user_to_insert.availability_schedule = Some(availability);
        }

        // Insert user
        let res = self
            .collection
            .insert_one(&user_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert user: {}", e),
            })?;

        let inserted_id = res.inserted_id.as_object_id().ok_or_else(|| AppError {
            message: "Failed to extract inserted_id".to_string(),
        })?;

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "User not found after insertion".to_string(),
            })
    }

    // =========================================================
    // 🔹 Get All Users (search + pagination)
    // =========================================================

    pub async fn get_all_users(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<PaginatedUsers, AppError> {
        // ✅ fixed: BaseRepository expects Collection<Document>
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable_fields = [
            "name",
            "email",
            "username",
            "phone",
            "role",
            "gender",
            "bio",
            "dream_career",
            "languages_spoken",
            "hobbies_interests",
            "special_skills",
            "department",
            "job_title",
            "preferred_communication_method",
        ];

        let (users, total, total_pages, current_page) = base_repo
            .get_all::<User>(filter, &searchable_fields, limit, skip, extra_match)
            .await?;

        Ok(PaginatedUsers {
            users,
            total,
            total_pages,
            current_page,
        })
    }

    // =========================================================
    // 🔹 Update User
    // =========================================================

    pub async fn update_user_fields(
        &self,
        id: &str,
        update_dto: &UpdateUserDto,
    ) -> Result<User, AppError> {
        let obj_id = Self::obj_id_from_str(id)?;

        let update_doc_full = bson::to_document(update_dto).map_err(|e| AppError {
            message: format!("Failed to serialize update dto: {}", e),
        })?;

        // ✅ Fix: bson::Document has no retain, so use filter_map
        let mut update_doc = Document::new();
        for (k, v) in update_doc_full {
            if !matches!(v, bson::Bson::Null) {
                update_doc.insert(k, v);
            }
        }

        if update_doc.is_empty() {
            return Err(AppError {
                message: "No valid fields to update".into(),
            });
        }

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        self.update_one_and_fetch(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
    }

    // =========================================================
    // 🔹 Delete User
    // =========================================================

    pub async fn delete_user(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = Self::obj_id_from_idtype(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete user: {}", e),
            })?;

        if result.deleted_count == 0 {
            Err(AppError {
                message: "No user deleted (user may not exist)".into(),
            })
        } else {
            Ok(())
        }
    }

    // =========================================================
    // 🔹 Statistics
    // =========================================================

    pub async fn get_user_stats(&self) -> Result<UserStats, AppError> {
        let count = |filter| async { self.collection.count_documents(filter).await.unwrap_or(0) };

        let total = count(doc! {}).await;
        let male = count(doc! { "gender": "MALE" }).await;
        let female = count(doc! { "gender": "FEMALE" }).await;
        let other = count(doc! { "gender": "OTHER" }).await;

        let admins = count(doc! { "role": "ADMIN" }).await;
        let staff = count(doc! { "role": "SCHOOLSTAFF" }).await;
        let students = count(doc! { "role": "STUDENT" }).await;
        let teachers = count(doc! { "role": "TEACHER" }).await;

        let assigned_school = count(doc! { "current_school_id": { "$exists": true } }).await;
        let no_school = count(doc! { "current_school_id": { "$exists": false } }).await;

        let thirty_days_ago = Utc::now() - Duration::days(30);
        let recent_30_days = count(doc! {
            "created_at": { "$gte": BsonDateTime::from_system_time(SystemTime::from(thirty_days_ago)) }
        })
        .await;

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
    // 🔹 Add / Remove Schools
    // =========================================================

    pub async fn add_school_to_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        let user_obj = Self::obj_id_from_idtype(user_id)?;
        let school_obj = Self::obj_id_from_idtype(school_id)?;
        let filter = doc! { "_id": &user_obj };

        // Ensure `schools` array exists
        self.collection
            .update_one(
                doc! { "_id": &user_obj, "$or": [ { "schools": { "$exists": false } }, { "schools": bson::Bson::Null } ] },
                doc! { "$set": { "schools": bson::to_bson(&Vec::<ObjectId>::new()).unwrap() } },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to initialize schools field: {}", e),
            })?;

        // Add school
        self.update_one_and_fetch(
            filter,
            doc! {
                "$addToSet": { "schools": &school_obj },
                "$set": {
                    "current_school_id": &school_obj,
                    "updated_at": bson::to_bson(&Utc::now()).unwrap()
                }
            },
        )
        .await
    }

    pub async fn remove_school_from_user(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<User, AppError> {
        let user_obj = Self::obj_id_from_idtype(user_id)?;
        let school_obj = Self::obj_id_from_idtype(school_id)?;

        self.update_one_and_fetch(
            doc! { "_id": &user_obj },
            doc! {
                "$pull": { "schools": &school_obj },
                "$set": { "updated_at": bson::to_bson(&Utc::now()).unwrap() }
            },
        )
        .await
    }
}
