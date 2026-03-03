use crate::domain::user::{PaginatedUsers, UpdateUserDto, User, UserStats};
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::models::mongo_model::IndexDef;
use crate::repositories::base_repo::BaseRepository;
use crate::services::user_service::normalize_user_ids;
use crate::utils::mongo_utils::extract_valid_fields;
use chrono::{Duration, Utc, Weekday};
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime as BsonDateTime, Document},
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

    // =========================================================
    // 🔹 Ensure Indexes (Performance Optimization)
    // =========================================================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            // Authentication & lookup indexes
            IndexDef::single("email", true),
            IndexDef::single_with_partial(
                "username",
                true,
                doc! { "username": { "$exists": true, "$ne": null } },
                Some("username_unique_idx"),
            ),
            // School relationship indexes
            IndexDef::single("current_school_id", false),
            IndexDef::compound(vec![("current_school_id", 1), ("role", 1)], false),
            // Role-based queries
            IndexDef::single("role", false),
            // Status and filtering
            IndexDef::single("disable", false),
            IndexDef::single("gender", false),
            // Timestamp indexes for sorting and filtering
            IndexDef::single_with_name("created_at", false, "created_at_desc", -1),
            IndexDef::single_with_name("updated_at", false, "updated_at_desc", -1),
            // School array index for membership queries
            IndexDef::single("schools", false),
            // Class access index
            IndexDef::single("accessible_classes", false),
            // Compound indexes for common queries
            IndexDef::compound(vec![("role", 1), ("current_school_id", 1)], false),
            IndexDef::compound(vec![("role", 1), ("disable", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================================================
    // 🔹 Utility Helpers
    // =========================================================

    fn obj_id_from_str(id: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(id).map_err(|e| AppError {
            message: format!("Invalid ObjectId: {}", e),
        })
    }

    pub async fn find_one_by_filter(&self, filter: Document) -> Result<Option<User>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<User>(filter, None).await
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
        let obj_id = IdType::to_object_id(id)?;
        self.find_one_by_filter(doc! { "_id": obj_id }).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<User, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<User>(filter, None)
            .await?
            .ok_or(AppError {
                message: "User not found".into(),
            })
    }

    // =========================================================
    // 🔹 Insert User
    // =========================================================

    pub async fn insert_user(&self, user: &mut User) -> Result<User, AppError> {
        // Ensure indexes exist
        self.ensure_indexes().await?;

        // Normalize and set timestamps
        let mut user_to_insert = normalize_user_ids(user.clone())?;
        user_to_insert.id = None;
        user_to_insert.created_at = Some(Utc::now());
        user_to_insert.updated_at = Some(Utc::now());

        // Default availability schedule
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

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Use BaseRepository create method
        let user_doc = bson::to_document(&user_to_insert).map_err(|e| AppError {
            message: format!("Failed to serialize user: {}", e),
        })?;

        repo.create::<User>(extract_valid_fields(user_doc), None)
            .await
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
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

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
            "_id",
            "current_school_id",
            "schools",
        ];

        let (users, total, total_pages, current_page) = repo
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

        // Filter out null values
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

    pub async fn update(
        &self,
        id: &IdType,
        update_dto: &UpdateUserDto,
    ) -> Result<User, AppError> {
        let update_doc_full = bson::to_document(update_dto).map_err(|e| AppError {
            message: format!("Failed to serialize update dto: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<User>(id, extract_valid_fields(update_doc_full))
            .await
    }

    // =========================================================
    // 🔹 Delete User
    // =========================================================

    pub async fn delete_user(&self, id: &IdType) -> Result<(), AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.delete_one(id).await
    }

    // Soft delete
    pub async fn soft_delete(&self, id: &IdType, deleted_by: ObjectId) -> Result<User, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let update_doc = doc! {
            "$set": {
                "deleted_at": bson::to_bson(&Utc::now()).unwrap(),
                "deleted_by": deleted_by,
                "disable": true
            }
        };

        repo.update_one_raw(id, update_doc).await?;
        self.find_one(Some(id), None).await
    }

    // Restore soft deleted user
    pub async fn restore(&self, id: &IdType) -> Result<User, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let update_doc = doc! {
            "$unset": {
                "deleted_at": "",
                "deleted_by": ""
            },
            "$set": {
                "disable": false,
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        repo.update_one_raw(id, update_doc).await?;
        self.find_one(Some(id), None).await
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
        let user_obj = IdType::to_object_id(user_id)?;
        let school_obj = IdType::to_object_id(school_id)?;
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
        let user_obj = IdType::to_object_id(user_id)?;
        let school_obj = IdType::to_object_id(school_id)?;

        self.update_one_and_fetch(
            doc! { "_id": &user_obj },
            doc! {
                "$pull": { "schools": &school_obj },
                "$set": { "updated_at": bson::to_bson(&Utc::now()).unwrap() }
            },
        )
        .await
    }

    // =========================================================
    // 🔹 Bulk Operations
    // =========================================================

    pub async fn create_many(&self, users: Vec<User>) -> Result<Vec<User>, AppError> {
        self.ensure_indexes().await?;

        let mut docs = Vec::new();
        for user in users {
            let mut user_to_insert = normalize_user_ids(user.clone())?;
            user_to_insert.id = None;
            user_to_insert.created_at = Some(Utc::now());
            user_to_insert.updated_at = Some(Utc::now());

            let doc = bson::to_document(&user_to_insert).map_err(|e| AppError {
                message: format!("Failed to serialize user: {}", e),
            })?;

            docs.push(extract_valid_fields(doc));
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create_many::<User>(docs, None).await
    }

    pub async fn update_many(
        &self,
        filter: Document,
        update_dto: &UpdateUserDto,
    ) -> Result<Vec<User>, AppError> {
        let update_doc_full = bson::to_document(update_dto).map_err(|e| AppError {
            message: format!("Failed to serialize update dto: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_many_and_fetch::<User>(filter, extract_valid_fields(update_doc_full))
            .await
    }
}
