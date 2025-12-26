use crate::domain::common_details::Paginated;
use crate::domain::school::{School, SchoolStats, UpdateSchool};
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::repositories::base_repo::BaseRepository;
use chrono::{Duration, Utc};
use mongodb::bson::Document;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime as BsonDateTime},
    options::IndexOptions,
    Collection, Database, IndexModel,
};
use std::time::SystemTime;

pub struct SchoolRepo {
    pub collection: Collection<School>,
}

impl SchoolRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<School>("schools"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let code_index = IndexModel::builder()
            .keys(doc! { "code": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.collection
            .create_indexes(vec![code_index, username_index])
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create indexes: {}", e),
            })?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<School>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse school id: {}", e),
        })?;

        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school by id: {}", e),
            })
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<School>, AppError> {
        self.collection
            .find_one(doc! { "code": code })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school by code: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<School>, AppError> {
        self.collection
            .find_one(doc! { "username": username })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school by username: {}", e),
            })
    }

    pub async fn get_all_schools(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<School>, AppError> {
        let searchable = [
            "name",
            "code",
            "description",
            "school_type",
            "username",
            "accreditation_number",
            "school_type",
            "school_members",
            "address",
            "website",
            "student_capacity",
            "classrooms",
            "affiliation",
            "_id",
        ];

        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let (data, total, total_pages, current_page) = base_repo
            .get_all::<School>(filter, &searchable, limit, skip, extra_match)
            .await?;
        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn insert_school(&self, school: &School) -> Result<School, AppError> {
        // Ensure indexes are created
        self.ensure_indexes().await?;

        let mut school_to_insert = school.clone();
        school_to_insert.id = None; // let MongoDB generate _id
        school_to_insert.created_at = Some(Utc::now());
        school_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&school_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert school: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: format!(
                    "Failed to convert inserted_id ({:?}) to ObjectId",
                    res.inserted_id
                ),
            })?
            .to_owned();

        match self.find_by_id(&IdType::from_object_id(inserted_id)).await {
            Ok(Some(s)) => Ok(s),
            Ok(None) => Err(AppError {
                message: "School not found".to_string(),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn update_school(
        &self,
        id: &IdType,
        updated_school: &School,
    ) -> Result<School, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(updated_school).map_err(|e| AppError {
            message: format!("Failed to convert school to document: {}", e),
        })?;
        update_doc.remove("_id");

        // Add updated_at timestamp
        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        self.collection
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update school: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "School not found after update".to_string(),
        })
    }

    pub async fn update_school_partial(
        &self,
        id: &IdType,
        update: UpdateSchool,
    ) -> Result<School, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(&update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        // Clean up fields: remove _id and null values
        update_doc = update_doc
            .into_iter()
            .filter(|(k, v)| k != "_id" && !matches!(v, bson::Bson::Null))
            .collect();

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        let update_query = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_query)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update school: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "School not found after update".to_string(),
        })
    }

    pub async fn delete_school(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete school: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No school deleted; it may not exist".to_string(),
            });
        }

        Ok(())
    }

    pub async fn get_school_stats(&self) -> Result<SchoolStats, AppError> {
        let total = self.collection.count_documents(doc! {}).await?;
        let public = self
            .collection
            .count_documents(doc! { "school_type": "public" })
            .await?;
        let private = self
            .collection
            .count_documents(doc! { "school_type": "private" })
            .await?;
        let active = self
            .collection
            .count_documents(doc! { "is_active": true })
            .await?;
        let inactive = self
            .collection
            .count_documents(doc! { "is_active": false })
            .await?;

        // Recent 30 days calculation (same pattern as user_repo)
        let thirty_days_ago_chrono = Utc::now() - Duration::days(30);
        let thirty_days_ago_system: SystemTime = thirty_days_ago_chrono.into();
        let thirty_days_ago_bson = BsonDateTime::from_system_time(thirty_days_ago_system);

        let recent_30_days_count = self
            .collection
            .count_documents(doc! { "created_at": { "$gte": thirty_days_ago_bson } })
            .await?;

        Ok(SchoolStats {
            total: total as i64,
            public: public as i64,
            private: private as i64,
            active: active as i64,
            inactive: inactive as i64,
            recent_30_days: recent_30_days_count as i64,
        })
    }
}
