use crate::domain::class::{Class, ClassWithOthers, ClassWithSchool, UpdateClass};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::pipeline::class_pipeline::{class_with_others_pipeline, class_with_school_pipeline};
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct ClassRepo {
    pub collection: Collection<Class>,
}

impl ClassRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Class>("classes"),
        }
    }

    pub async fn get_all_with_school(&self) -> Result<Vec<ClassWithSchool>, AppError> {
        aggregate_many(&self.collection.clone().clone_with_type::<Document>(), {
            let mut pipeline = class_with_school_pipeline(doc! {});
            pipeline.insert(0, doc! { "$sort": { "updated_at": -1 } });
            pipeline
        })
        .await
    }

    pub async fn find_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<Option<ClassWithOthers>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            class_with_others_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<Option<ClassWithOthers>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            class_with_others_pipeline(doc! { "username": username }),
        )
        .await
    }

    pub async fn find_by_code_with_others(
        &self,
        code: &str,
    ) -> Result<Option<ClassWithOthers>, AppError> {
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            class_with_others_pipeline(doc! { "code": code }),
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<Class>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find class by id: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<Class>, AppError> {
        self.collection
            .find_one(doc! { "username": username })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find class by username: {}", e),
            })
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<Class>, AppError> {
        self.collection
            .find_one(doc! { "code": code })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find class by code: {}", e),
            })
    }

    pub async fn find_by_school_id(&self, school_id: &IdType) -> Result<Vec<Class>, AppError> {
        let obj_id = parse_object_id(school_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "school_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find classes by school_id: {}", e),
            })?;

        let mut classes = Vec::new();
        while let Some(class) = cursor.next().await {
            classes.push(class.map_err(|e| AppError {
                message: format!("Failed to process class: {}", e),
            })?);
        }
        Ok(classes)
    }

    pub async fn find_by_creator_id(&self, creator_id: &IdType) -> Result<Vec<Class>, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find classes by creator_id: {}", e),
            })?;

        let mut classes = Vec::new();
        while let Some(class) = cursor.next().await {
            classes.push(class.map_err(|e| AppError {
                message: format!("Failed to process class: {}", e),
            })?);
        }
        Ok(classes)
    }

    pub async fn find_by_class_teacher_id(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<Class>, AppError> {
        let obj_id = parse_object_id(teacher_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "class_teacher_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find classes by class_teacher_id: {}", e),
            })?;

        let mut classes = Vec::new();
        while let Some(class) = cursor.next().await {
            classes.push(class.map_err(|e| AppError {
                message: format!("Failed to process class: {}", e),
            })?);
        }
        Ok(classes)
    }

    pub async fn find_by_main_class_id(
        &self,
        main_class_id: &IdType,
    ) -> Result<Vec<Class>, AppError> {
        let obj_id = parse_object_id(main_class_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "main_class_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find classes by main_class_id: {}", e),
            })?;

        let mut classes = Vec::new();
        while let Some(class) = cursor.next().await {
            classes.push(class.map_err(|e| AppError {
                message: format!("Failed to process class: {}", e),
            })?);
        }
        Ok(classes)
    }

    pub async fn insert_class(&self, class: &Class) -> Result<Class, AppError> {
        self.ensure_indexes().await?;

        let mut to_insert = class.clone();
        to_insert.id = None;
        to_insert.created_at = Utc::now();
        to_insert.updated_at = Utc::now();

        let res = self
            .collection
            .insert_one(&to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert class: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted class id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Class not found after insert".to_string(),
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

        let school_index = IndexModel::builder()
            .keys(doc! { "school_id": 1 })
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

        let main_class_index = IndexModel::builder()
            .keys(doc! { "main_class_id": 1 })
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
            .create_index(school_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create school_id index: {}", e),
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
            .create_index(main_class_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create main_class_id index: {}", e),
            })?;

        Ok(())
    }

    pub async fn get_all_classes(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Class>, AppError> {
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
                        { "subject": &regex },
                        { "grade_level": &regex },
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
                message: format!("Failed to fetch classes: {}", e),
            })?;

        let mut classes = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate classes: {}", e),
        })? {
            let class: Class = mongodb::bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize class: {}", e),
            })?;
            classes.push(class);
        }

        Ok(classes)
    }

    pub async fn get_active_classes(&self) -> Result<Vec<Class>, AppError> {
        let mut cursor = self
            .collection
            .find(doc! { "is_active": true })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find active classes: {}", e),
            })?;

        let mut classes = Vec::new();
        while let Some(class) = cursor.next().await {
            classes.push(class.map_err(|e| AppError {
                message: format!("Failed to process class: {}", e),
            })?);
        }
        Ok(classes)
    }

    pub async fn update_class(&self, id: &IdType, update: &UpdateClass) -> Result<Class, AppError> {
        let obj_id = parse_object_id(id)?;

        // Create update document manually to handle Option<Option<T>> fields
        let mut update_doc = Document::new();

        if let Some(name) = &update.name {
            update_doc.insert("name", name);
        }
        if let Some(username) = &update.username {
            update_doc.insert("username", username);
        }
        if let Some(code) = &update.code {
            update_doc.insert(
                "code",
                bson::to_bson(code).map_err(|e| AppError {
                    message: format!("Failed to serialize code: {}", e),
                })?,
            );
        }
        if let Some(school_id) = &update.school_id {
            update_doc.insert(
                "school_id",
                bson::to_bson(school_id).map_err(|e| AppError {
                    message: format!("Failed to serialize school_id: {}", e),
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
        if let Some(r#type) = &update.r#type {
            update_doc.insert(
                "type",
                bson::to_bson(r#type).map_err(|e| AppError {
                    message: format!("Failed to serialize type: {}", e),
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
        if let Some(capacity) = update.capacity {
            update_doc.insert("capacity", capacity);
        }
        if let Some(subject) = &update.subject {
            update_doc.insert(
                "subject",
                bson::to_bson(subject).map_err(|e| AppError {
                    message: format!("Failed to serialize subject: {}", e),
                })?,
            );
        }
        if let Some(grade_level) = &update.grade_level {
            update_doc.insert(
                "grade_level",
                bson::to_bson(grade_level).map_err(|e| AppError {
                    message: format!("Failed to serialize grade_level: {}", e),
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
                message: format!("Failed to update class: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Class not found after update".to_string(),
        })
    }

    pub async fn delete_class(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete class: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No class deleted; it may not exist".to_string(),
            });
        }
        Ok(())
    }

    pub async fn count_by_school_id(&self, school_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(school_id)?;
        self.collection
            .count_documents(doc! { "school_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count classes by school_id: {}", e),
            })
    }

    pub async fn count_by_creator_id(&self, creator_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        self.collection
            .count_documents(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count classes by creator_id: {}", e),
            })
    }

    pub async fn create_many_classes(&self, classes: Vec<Class>) -> Result<Vec<Class>, AppError> {
        self.ensure_indexes().await?;

        let mut classes_to_insert = Vec::with_capacity(classes.len());
        let now = Utc::now();

        for mut class in classes {
            class.id = None;
            class.created_at = now;
            class.updated_at = now;
            classes_to_insert.push(class);
        }

        // Insert all classes
        let result = self
            .collection
            .insert_many(&classes_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert multiple classes: {}", e),
            })?;

        // ✅ Fix: use cloned() inside filter_map, not after
        let inserted_ids: Vec<ObjectId> = result
            .inserted_ids
            .values()
            .filter_map(|bson| bson.as_object_id())
            .collect();

        if inserted_ids.len() != classes_to_insert.len() {
            return Err(AppError {
                message: "Failed to get all inserted class IDs".to_string(),
            });
        }

        // Fetch and return the inserted classes
        let mut inserted_classes = Vec::with_capacity(inserted_ids.len());
        for id in inserted_ids {
            let class = self.find_by_id(&IdType::from_object_id(id)).await?;
            if let Some(class) = class {
                inserted_classes.push(class);
            }
        }

        Ok(inserted_classes)
    }

    /// Create multiple classes with validation and conflict checking
    pub async fn create_many_classes_with_validation(
        &self,
        classes: Vec<Class>,
    ) -> Result<Vec<Class>, AppError> {
        self.ensure_indexes().await?;

        // Check for duplicate usernames and codes in the input
        let mut usernames = std::collections::HashSet::new();
        let mut codes = std::collections::HashSet::new();

        for class in &classes {
            if !usernames.insert(&class.username) {
                return Err(AppError {
                    message: format!("Duplicate username found: {}", class.username),
                });
            }

            if let Some(code) = &class.code {
                if !codes.insert(code) {
                    return Err(AppError {
                        message: format!("Duplicate code found: {}", code),
                    });
                }
            }
        }

        // Check for existing usernames in database
        for class in &classes {
            if let Some(existing) = self.find_by_username(&class.username).await? {
                return Err(AppError {
                    message: format!("Username already exists: {}", existing.username),
                });
            }

            if let Some(code) = &class.code {
                if let Some(existing) = self.find_by_code(code).await? {
                    return Err(AppError {
                        message: format!("Code already exists: {:?}", existing.code),
                    });
                }
            }
        }

        // If all checks pass, create the classes
        self.create_many_classes(classes).await
    }

    /// Create multiple classes for a specific school
    pub async fn create_many_classes_for_school(
        &self,
        school_id: &IdType,
        classes: Vec<Class>,
    ) -> Result<Vec<Class>, AppError> {
        let obj_id = parse_object_id(school_id)?;

        let mut classes_with_school = Vec::with_capacity(classes.len());
        for mut class in classes {
            class.school_id = Some(obj_id);
            classes_with_school.push(class);
        }

        self.create_many_classes_with_validation(classes_with_school)
            .await
    }

    /// Bulk update multiple classes
    pub async fn update_many_classes(
        &self,
        updates: Vec<(IdType, UpdateClass)>,
    ) -> Result<Vec<Class>, AppError> {
        let mut updated_classes = Vec::with_capacity(updates.len());

        for (id, update) in updates {
            match self.update_class(&id, &update).await {
                Ok(class) => updated_classes.push(class),
                Err(e) => {
                    // Log the error but continue with other updates
                    eprintln!("Failed to update class {:?}: {}", id, e.message);
                }
            }
        }

        if updated_classes.is_empty() {
            return Err(AppError {
                message: "No classes were successfully updated".to_string(),
            });
        }

        Ok(updated_classes)
    }
}
