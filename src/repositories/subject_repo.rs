use crate::{
    domain::{
        subject::{Subject, SubjectWithRelations, UpdateSubject},
        subjects::subject_category::SubjectCategory,
    },
    errors::AppError,
    helpers::aggregate_helpers::{aggregate_many, aggregate_single},
    models::id_model::IdType,
    pipeline::subject_pipeline::subject_with_relations_pipeline,
    utils::object_id::parse_object_id,
};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct SubjectRepo {
    pub collection: Collection<Subject>,
}

impl SubjectRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Subject>("subjects"),
        }
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<SubjectWithRelations>, AppError> {
        let mut pipeline = vec![];

        // 🔍 Add search/filter functionality
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i" // case-insensitive search
            };

            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "username": &regex },
                        { "code": &regex },
                        { "description": &regex },
                        { "tags": &regex },
                    ]
                }
            });
        }

        // 🧩 Merge with subject relations pipeline
        let mut relations_pipeline = subject_with_relations_pipeline(doc! {});
        pipeline.append(&mut relations_pipeline);

        // 🕒 Sort by most recently updated
        pipeline.insert(0, doc! { "$sort": { "updated_at": -1 } });

        // ⏭️ Add pagination
        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }

        if let Some(l) = limit {
            pipeline.push(doc! { "$limit": l });
        } else {
            pipeline.push(doc! { "$limit": 50 }); // default limit
        }

        // 🧠 Run aggregation
        let docs = aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            pipeline,
        )
        .await?;

        Ok(docs)
    }

    pub async fn find_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<Option<SubjectWithRelations>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            subject_with_relations_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_username_with_relations(
        &self,
        username: &str,
    ) -> Result<Option<SubjectWithRelations>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            subject_with_relations_pipeline(doc! { "username": username }),
        )
        .await
    }

    pub async fn find_by_code_with_relations(
        &self,
        code: &str,
    ) -> Result<Option<SubjectWithRelations>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            subject_with_relations_pipeline(doc! { "code": code }),
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<Subject>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subject by id: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<Subject>, AppError> {
        self.collection
            .find_one(doc! { "username": username })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subject by username: {}", e),
            })
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<Subject>, AppError> {
        self.collection
            .find_one(doc! { "code": code })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subject by code: {}", e),
            })
    }

    pub async fn find_by_class_id(&self, class_id: &IdType) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(class_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "class_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subjects by class_id: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    pub async fn find_by_creator_id(&self, creator_id: &IdType) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subjects by creator_id: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    pub async fn find_by_class_teacher_id(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(teacher_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "class_teacher_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subjects by class_teacher_id: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    pub async fn find_by_class_teacher_id_with_relations(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<SubjectWithRelations>, AppError> {
        let obj_id = parse_object_id(teacher_id)?;

        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            subject_with_relations_pipeline(doc! {
                "class_teacher_id": obj_id
            }),
        )
        .await
    }

    pub async fn find_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(main_subject_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "main_subject_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subjects by main_subject_id: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    pub async fn find_by_subject_type(
        &self,
        subject_type: &SubjectCategory,
    ) -> Result<Vec<Subject>, AppError> {
        let mut cursor = self
            .collection
            .find(
                doc! { "subject_type": bson::to_bson(subject_type).map_err(|e| AppError {
                    message: format!("Failed to serialize subject_type: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subjects by subject_type: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    pub async fn insert_subject(&self, subject: &Subject) -> Result<Subject, AppError> {
        self.ensure_indexes().await?;

        let mut to_insert = subject.clone();
        to_insert.id = None;
        to_insert.created_at = Utc::now();
        to_insert.updated_at = Utc::now();

        let res = self
            .collection
            .insert_one(&to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert subject: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted subject id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Subject not found after insert".to_string(),
            })
    }

    async fn ensure_indexes(&self) -> Result<(), AppError> {
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let code_index = IndexModel::builder()
            .keys(doc! { "code": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let class_index = IndexModel::builder()
            .keys(doc! { "class_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let creator_index = IndexModel::builder()
            .keys(doc! { "creator_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let class_teacher_index = IndexModel::builder()
            .keys(doc! { "class_teacher_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let main_subject_index = IndexModel::builder()
            .keys(doc! { "main_subject_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let subject_type_index = IndexModel::builder()
            .keys(doc! { "subject_type": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        self.collection
            .create_index(username_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create username index: {}", e),
            })?;

        self.collection
            .create_index(code_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create code index: {}", e),
            })?;

        self.collection
            .create_index(class_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create class_id index: {}", e),
            })?;

        self.collection
            .create_index(creator_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create creator_id index: {}", e),
            })?;

        self.collection
            .create_index(class_teacher_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create class_teacher_id index: {}", e),
            })?;

        self.collection
            .create_index(main_subject_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create main_subject_id index: {}", e),
            })?;

        self.collection
            .create_index(subject_type_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create subject_type index: {}", e),
            })?;

        Ok(())
    }

    pub async fn get_all_subjects(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Subject>, AppError> {
        let mut pipeline = vec![];

        // Add search/filter functionality
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"  // case insensitive
            };
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "username": &regex },
                        { "code": &regex },
                        { "description": &regex },
                        { "tags": &regex },
                    ]
                }
            });
        }

        // Add sorting by updated_at (most recent first)
        pipeline.push(doc! { "$sort": { "updated_at": -1 } });

        // Add pagination
        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }

        if let Some(l) = limit {
            pipeline.push(doc! { "$limit": l });
        } else {
            // Default limit if none provided
            pipeline.push(doc! { "$limit": 50 });
        }

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch subjects: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate subjects: {}", e),
        })? {
            let subject: Subject = mongodb::bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize subject: {}", e),
            })?;
            subjects.push(subject);
        }

        Ok(subjects)
    }

    pub async fn get_active_subjects(&self) -> Result<Vec<Subject>, AppError> {
        let mut cursor = self
            .collection
            .find(doc! { "is_active": true })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find active subjects: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    pub async fn update_subject(
        &self,
        id: &IdType,
        update: &UpdateSubject,
    ) -> Result<Subject, AppError> {
        let obj_id = parse_object_id(id)?;

        // Create update document manually to handle Option<Option<T>> fields
        let mut update_doc = Document::new();

        if let Some(name) = &update.name {
            update_doc.insert("name", name);
        }
        if let Some(username) = &update.username {
            update_doc.insert("username", username);
        }
        if let Some(class_id) = &update.class_id {
            update_doc.insert(
                "class_id",
                bson::to_bson(class_id).map_err(|e| AppError {
                    message: format!("Failed to serialize class_id: {}", e),
                })?,
            );
        }
        if let Some(class_teacher_id) = &update.class_teacher_id {
            update_doc.insert(
                "class_teacher_id",
                bson::to_bson(class_teacher_id).map_err(|e| AppError {
                    message: format!("Failed to serialize class_teacher_id: {}", e),
                })?,
            );
        }
        if let Some(main_subject_id) = &update.main_subject_id {
            update_doc.insert(
                "main_subject_id",
                bson::to_bson(main_subject_id).map_err(|e| AppError {
                    message: format!("Failed to serialize main_subject_id: {}", e),
                })?,
            );
        }
        if let Some(subject_type) = &update.subject_type {
            update_doc.insert(
                "subject_type",
                bson::to_bson(subject_type).map_err(|e| AppError {
                    message: format!("Failed to serialize subject_type: {}", e),
                })?,
            );
        }
        if let Some(is_active) = update.is_active {
            update_doc.insert("is_active", is_active);
        }
        if let Some(description) = &update.description {
            update_doc.insert(
                "description",
                bson::to_bson(description).map_err(|e| AppError {
                    message: format!("Failed to serialize description: {}", e),
                })?,
            );
        }
        if let Some(code) = &update.code {
            update_doc.insert(
                "code",
                bson::to_bson(code).map_err(|e| AppError {
                    message: format!("Failed to serialize code: {}", e),
                })?,
            );
        }
        if let Some(tags) = &update.tags {
            update_doc.insert("tags", tags);
        }

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        let update_doc = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update subject: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Subject not found after update".to_string(),
        })
    }

    pub async fn delete_subject(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete subject: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No subject deleted; it may not exist".to_string(),
            });
        }
        Ok(())
    }

    pub async fn count_by_class_id(&self, class_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(class_id)?;
        self.collection
            .count_documents(doc! { "class_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count subjects by class_id: {}", e),
            })
    }

    pub async fn count_by_creator_id(&self, creator_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        self.collection
            .count_documents(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count subjects by creator_id: {}", e),
            })
    }

    pub async fn count_by_class_teacher_id(&self, teacher_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(teacher_id)?;
        self.collection
            .count_documents(doc! { "class_teacher_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count subjects by class_teacher_id: {}", e),
            })
    }

    pub async fn count_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<u64, AppError> {
        let obj_id = parse_object_id(main_subject_id)?;
        self.collection
            .count_documents(doc! { "main_subject_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count subjects by main_subject_id: {}", e),
            })
    }

    pub async fn create_many_subjects(
        &self,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, AppError> {
        self.ensure_indexes().await?;

        let mut subjects_to_insert = Vec::with_capacity(subjects.len());
        let now = Utc::now();

        for mut subject in subjects {
            subject.id = None;
            subject.created_at = now;
            subject.updated_at = now;
            subjects_to_insert.push(subject);
        }

        // Insert all subjects
        let result = self
            .collection
            .insert_many(&subjects_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert multiple subjects: {}", e),
            })?;

        // Get the inserted IDs
        let inserted_ids: Vec<ObjectId> = result
            .inserted_ids
            .values()
            .filter_map(|bson| bson.as_object_id().map(|oid| oid.clone()))
            .collect();

        if inserted_ids.len() != subjects_to_insert.len() {
            return Err(AppError {
                message: "Failed to get all inserted subject IDs".to_string(),
            });
        }

        // Fetch and return the inserted subjects
        let mut inserted_subjects = Vec::with_capacity(inserted_ids.len());
        for id in inserted_ids {
            let subject = self.find_by_id(&IdType::from_object_id(id)).await?;
            if let Some(subject) = subject {
                inserted_subjects.push(subject);
            }
        }

        Ok(inserted_subjects)
    }

    /// Create multiple subjects with validation and conflict checking
    pub async fn create_many_subjects_with_validation(
        &self,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, AppError> {
        self.ensure_indexes().await?;

        // Check for duplicate usernames and codes in the input
        let mut usernames = std::collections::HashSet::new();
        let mut codes = std::collections::HashSet::new();

        for subject in &subjects {
            if !usernames.insert(&subject.username) {
                return Err(AppError {
                    message: format!("Duplicate username found: {}", subject.username),
                });
            }

            if let Some(code) = &subject.code {
                if !codes.insert(code) {
                    return Err(AppError {
                        message: format!("Duplicate code found: {}", code),
                    });
                }
            }
        }

        // Check for existing usernames in database
        for subject in &subjects {
            if let Some(existing) = self.find_by_username(&subject.username).await? {
                return Err(AppError {
                    message: format!("Username already exists: {}", existing.username),
                });
            }

            if let Some(code) = &subject.code {
                if let Some(existing) = self.find_by_code(code).await? {
                    return Err(AppError {
                        message: format!("Code already exists: {:?}", existing.code),
                    });
                }
            }
        }

        // If all checks pass, create the subjects
        self.create_many_subjects(subjects).await
    }

    /// Create multiple subjects for a specific class
    pub async fn create_many_subjects_for_class(
        &self,
        class_id: &IdType,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(class_id)?;

        let mut subjects_with_class = Vec::with_capacity(subjects.len());
        for mut subject in subjects {
            subject.class_id = Some(obj_id);
            subjects_with_class.push(subject);
        }

        self.create_many_subjects_with_validation(subjects_with_class)
            .await
    }

    /// Create multiple subjects for a specific teacher
    pub async fn create_many_subjects_for_teacher(
        &self,
        teacher_id: &IdType,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(teacher_id)?;

        let mut subjects_with_teacher = Vec::with_capacity(subjects.len());
        for mut subject in subjects {
            subject.class_teacher_id = Some(obj_id);
            subjects_with_teacher.push(subject);
        }

        self.create_many_subjects_with_validation(subjects_with_teacher)
            .await
    }

    /// Create multiple subjects for a specific main subject
    pub async fn create_many_subjects_for_main_subject(
        &self,
        main_subject_id: &IdType,
        subjects: Vec<Subject>,
    ) -> Result<Vec<Subject>, AppError> {
        let obj_id = parse_object_id(main_subject_id)?;

        let mut subjects_with_main = Vec::with_capacity(subjects.len());
        for mut subject in subjects {
            subject.main_subject_id = Some(obj_id);
            subjects_with_main.push(subject);
        }

        self.create_many_subjects_with_validation(subjects_with_main)
            .await
    }

    /// Bulk update multiple subjects
    pub async fn update_many_subjects(
        &self,
        updates: Vec<(IdType, UpdateSubject)>,
    ) -> Result<Vec<Subject>, AppError> {
        let mut updated_subjects = Vec::with_capacity(updates.len());

        for (id, update) in updates {
            match self.update_subject(&id, &update).await {
                Ok(subject) => updated_subjects.push(subject),
                Err(e) => {
                    // Log the error but continue with other updates
                    eprintln!("Failed to update subject {:?}: {}", id, e.message);
                }
            }
        }

        if updated_subjects.is_empty() {
            return Err(AppError {
                message: "No subjects were successfully updated".to_string(),
            });
        }

        Ok(updated_subjects)
    }

    /// Check if usernames or codes already exist in the database
    pub async fn check_existing_identifiers(
        &self,
        usernames: &[String],
        codes: &[String],
    ) -> Result<(Vec<String>, Vec<String>), AppError> {
        let mut existing_usernames = Vec::new();
        let mut existing_codes = Vec::new();

        // Check usernames
        for username in usernames {
            if (self.find_by_username(username).await?).is_some() {
                existing_usernames.push(username.clone());
            }
        }

        // Check codes
        for code in codes {
            if let Some(_) = self.find_by_code(code).await? {
                existing_codes.push(code.clone());
            }
        }

        Ok((existing_usernames, existing_codes))
    }

    /// Bulk delete multiple subjects
    pub async fn delete_many_subjects(&self, ids: Vec<IdType>) -> Result<u64, AppError> {
        let object_ids: Result<Vec<ObjectId>, AppError> = ids.iter().map(parse_object_id).collect();

        let object_ids = object_ids?;

        let filter = doc! {
            "_id": {
                "$in": object_ids
            }
        };

        let result = self
            .collection
            .delete_many(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete multiple subjects: {}", e),
            })?;

        Ok(result.deleted_count)
    }

    /// Get subjects by multiple IDs
    pub async fn find_by_ids(&self, ids: Vec<IdType>) -> Result<Vec<Subject>, AppError> {
        let object_ids: Result<Vec<ObjectId>, AppError> =
            ids.iter().map(|id| parse_object_id(id)).collect();

        let object_ids = object_ids?;

        let mut cursor = self
            .collection
            .find(doc! {
                "_id": {
                    "$in": object_ids
                }
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subjects by IDs: {}", e),
            })?;

        let mut subjects = Vec::new();
        while let Some(subject) = cursor.next().await {
            subjects.push(subject.map_err(|e| AppError {
                message: format!("Failed to process subject: {}", e),
            })?);
        }
        Ok(subjects)
    }

    /// Get subjects with relations by multiple IDs
    pub async fn find_by_ids_with_relations(
        &self,
        ids: Vec<IdType>,
    ) -> Result<Vec<SubjectWithRelations>, AppError> {
        let object_ids: Result<Vec<ObjectId>, AppError> = ids.iter().map(parse_object_id).collect();

        let object_ids = object_ids?;

        aggregate_many(
            &self.collection.clone().clone_with_type::<Document>(),
            subject_with_relations_pipeline(doc! {
                "_id": {
                    "$in": object_ids
                }
            }),
        )
        .await
    }

    /// Bulk update subjects by class ID
    pub async fn update_many_by_class_id(
        &self,
        class_id: &IdType,
        update: &UpdateSubject,
    ) -> Result<u64, AppError> {
        let obj_id = parse_object_id(class_id)?;

        let mut update_doc = Document::new();

        // Add fields to update
        if let Some(name) = &update.name {
            update_doc.insert("name", name);
        }
        if let Some(username) = &update.username {
            update_doc.insert("username", username);
        }
        if let Some(class_teacher_id) = &update.class_teacher_id {
            update_doc.insert(
                "class_teacher_id",
                bson::to_bson(class_teacher_id).map_err(|e| AppError {
                    message: format!("Failed to serialize class_teacher_id: {}", e),
                })?,
            );
        }
        if let Some(main_subject_id) = &update.main_subject_id {
            update_doc.insert(
                "main_subject_id",
                bson::to_bson(main_subject_id).map_err(|e| AppError {
                    message: format!("Failed to serialize main_subject_id: {}", e),
                })?,
            );
        }
        if let Some(subject_type) = &update.subject_type {
            update_doc.insert(
                "subject_type",
                bson::to_bson(subject_type).map_err(|e| AppError {
                    message: format!("Failed to serialize subject_type: {}", e),
                })?,
            );
        }
        if let Some(is_active) = update.is_active {
            update_doc.insert("is_active", is_active);
        }
        if let Some(description) = &update.description {
            update_doc.insert(
                "description",
                bson::to_bson(description).map_err(|e| AppError {
                    message: format!("Failed to serialize description: {}", e),
                })?,
            );
        }
        if let Some(code) = &update.code {
            update_doc.insert(
                "code",
                bson::to_bson(code).map_err(|e| AppError {
                    message: format!("Failed to serialize code: {}", e),
                })?,
            );
        }
        if let Some(tags) = &update.tags {
            update_doc.insert("tags", tags);
        }

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        let update_doc = doc! { "$set": update_doc };

        let result = self
            .collection
            .update_many(doc! { "class_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update subjects by class_id: {}", e),
            })?;

        Ok(result.modified_count)
    }

    /// Bulk activate/deactivate subjects
    pub async fn bulk_update_active_status(
        &self,
        ids: Vec<IdType>,
        is_active: bool,
    ) -> Result<u64, AppError> {
        let object_ids: Result<Vec<ObjectId>, AppError> =
            ids.iter().map(|id| parse_object_id(id)).collect();

        let object_ids = object_ids?;

        let update_doc = doc! {
            "$set": {
                "is_active": is_active,
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        let result = self
            .collection
            .update_many(
                doc! {
                    "_id": {
                        "$in": object_ids
                    }
                },
                update_doc,
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk update active status: {}", e),
            })?;

        Ok(result.modified_count)
    }
}
