use crate::domain::class::{Class, ClassLevelType, ClassWithOthers, PaginatedClasses, UpdateClass};
use crate::domain::main_class::MainClass;
use crate::domain::school::School;
use crate::domain::teacher::Teacher;
use crate::domain::trade::Trade;
use crate::domain::user::User;
use crate::errors::AppError;
use crate::models::id_model::IdType;
use crate::pipeline::class_pipeline::class_with_others_pipeline;
use crate::repositories::base_repo::BaseRepository;
use crate::utils::object_id::parse_object_id;
use crate::utils::school_utils::sanitize_school;
use crate::utils::user_utils::sanitize_user;

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

    pub async fn find_class_with_others(
        &self,
        filter: Option<Document>,
        search: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        // --- Build match filter ---
        let mut query = filter.unwrap_or_else(|| doc! {});

        // Convert possible string _id filters to ObjectId
        if let Ok(school_id) = query.get_str("school_id") {
            if let Ok(obj_id) = ObjectId::parse_str(school_id) {
                query.insert("school_id", obj_id);
            }
        }
        if let Ok(creator_id) = query.get_str("creator_id") {
            if let Ok(obj_id) = ObjectId::parse_str(creator_id) {
                query.insert("creator_id", obj_id);
            }
        }
        if let Ok(class_teacher_id) = query.get_str("class_teacher_id") {
            if let Ok(obj_id) = ObjectId::parse_str(class_teacher_id) {
                query.insert("class_teacher_id", obj_id);
            }
        }
        if let Ok(main_class_id) = query.get_str("main_class_id") {
            if let Ok(obj_id) = ObjectId::parse_str(main_class_id) {
                query.insert("main_class_id", obj_id);
            }
        }

        // --- Add search ---
        if let Some(search_text) = search {
            let regex = doc! { "$regex": search_text, "$options": "i" };
            query.insert(
                "$or",
                vec![
                    doc! { "name": &regex },
                    doc! { "username": &regex },
                    doc! { "code": &regex },
                    doc! { "description": &regex },
                    doc! { "subject": &regex },
                ],
            );
        }

        // --- Build pipeline ---
        let mut pipeline = class_with_others_pipeline(query);
        if let Some(skip_val) = skip {
            pipeline.push(doc! { "$skip": skip_val });
        }
        if let Some(limit_val) = limit {
            pipeline.push(doc! { "$limit": limit_val });
        }

        // --- Execute aggregation ---
        let mut cursor = self
            .collection
            .clone_with_type::<Document>()
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to aggregate classes with others: {}", e),
            })?;

        let mut results = Vec::new();

        // --- Iterate through results ---
        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to process aggregated class: {}", e),
        })? {
            let mut base_doc = doc.clone();
            base_doc.remove("school");
            base_doc.remove("creator");
            base_doc.remove("class_teacher");
            base_doc.remove("main_class");

            let class: Class = mongodb::bson::from_document(base_doc).map_err(|e| AppError {
                message: format!("Failed to deserialize class: {}", e),
            })?;

            let mut school = doc
                .get_document("school")
                .ok()
                .and_then(|d| mongodb::bson::from_document::<School>(d.clone()).ok())
                .map(sanitize_school);
            if let Some(s) = school {
                school = Some(sanitize_school(s));
            }

            let mut creator = doc
                .get_document("creator")
                .ok()
                .and_then(|d| mongodb::bson::from_document::<User>(d.clone()).ok())
                .map(sanitize_user);
            if let Some(c) = creator {
                creator = Some(sanitize_user(c));
            }

            let trade = doc
                .get_document("trade")
                .ok()
                .and_then(|d| mongodb::bson::from_document::<Trade>(d.clone()).ok());

            let class_teacher = doc
                .get_document("class_teacher")
                .ok()
                .and_then(|d| mongodb::bson::from_document::<Teacher>(d.clone()).ok());

            let main_class = doc
                .get_document("main_class")
                .ok()
                .and_then(|d| mongodb::bson::from_document::<MainClass>(d.clone()).ok());

            results.push(ClassWithOthers {
                class,
                school,
                creator,
                class_teacher,
                main_class,
                trade,
            });
        }

        Ok(results)
    }

    pub async fn find_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<Option<ClassWithOthers>, AppError> {
        let obj_id = parse_object_id(id)?;
        let results = self
            .find_class_with_others(
                Some(doc! { "_id": obj_id }),
                None,    // no search text
                Some(1), // limit to one
                None,    // no skip
            )
            .await?;

        Ok(results.into_iter().next())
    }

    pub async fn find_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<Option<ClassWithOthers>, AppError> {
        let results = self
            .find_class_with_others(
                Some(doc! { "username": username }),
                None,    // no search
                Some(1), // limit to one
                None,    // no skip
            )
            .await?;

        Ok(results.into_iter().next())
    }

    pub async fn find_by_code_with_others(
        &self,
        code: &str,
    ) -> Result<Option<ClassWithOthers>, AppError> {
        let results = self
            .find_class_with_others(
                Some(doc! { "code": code }),
                None,    // no search
                Some(1), // limit to one
                None,    // no skip
            )
            .await?;

        Ok(results.into_iter().next())
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
        extra_match: Option<Document>, // 👈 extra filter support
    ) -> Result<PaginatedClasses, AppError> {
        // ✅ Use BaseRepository with correct type (Document)
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Define which fields to search in
        let searchable_fields = [
            "name",
            "username",
            "code",
            "description",
            "subject",
            "grade_level",
            "tags",
        ];

        // ✅ Call the generic get_all<T>() method
        let (classes, total, total_pages, current_page) = base_repo
            .get_all::<Class>(filter, &searchable_fields, limit, skip, extra_match)
            .await?;

        Ok(PaginatedClasses {
            classes,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update_class(&self, id: &IdType, update: &UpdateClass) -> Result<Class, AppError> {
        let obj_id = parse_object_id(id)?;

        let mut update_doc = Document::new();

        macro_rules! insert_if_some {
            ($field:expr, $name:expr) => {
                if let Some(value) = &$field {
                    update_doc.insert(
                        $name,
                        bson::to_bson(value).map_err(|e| AppError {
                            message: format!("Failed to serialize {}: {}", $name, e),
                        })?,
                    );
                }
            };
        }

        insert_if_some!(update.name, "name");
        insert_if_some!(update.username, "username");
        insert_if_some!(update.code, "code");
        insert_if_some!(update.school_id, "school_id");
        insert_if_some!(update.r#type, "type");
        insert_if_some!(update.description, "description");
        insert_if_some!(update.subject, "subject");
        insert_if_some!(update.grade_level, "grade_level");
        insert_if_some!(update.tags, "tags");
        insert_if_some!(update.image_id, "image_id");
        insert_if_some!(update.image, "image");
        insert_if_some!(update.background_images, "background_images");

        if let Some(is_active) = update.is_active {
            update_doc.insert("is_active", is_active);
        }
        if let Some(capacity) = update.capacity {
            update_doc.insert("capacity", capacity);
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
            message: "Class not found after update".into(),
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

    pub async fn find_many_by_ids(&self, ids: Vec<ObjectId>) -> Result<Vec<Class>, AppError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let filter = doc! {
            "_id": { "$in": ids }
        };

        let mut cursor = self.collection.find(filter).await.map_err(|e| AppError {
            message: format!("Failed to query classes by IDs: {}", e),
        })?;

        let mut classes = Vec::new();
        while let Some(class) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Error reading class: {}", e),
        })? {
            classes.push(class);
        }

        Ok(classes)
    }

    pub async fn add_or_update_class_teacher(
        &self,
        class_id: &IdType,
        teacher_id: &IdType,
    ) -> Result<Class, AppError> {
        // Try to find the class first
        if let Some(mut existing_class) = self.find_by_id(class_id).await? {
            let cls_id = parse_object_id(class_id)?;
            let tea_id = parse_object_id(teacher_id)?;

            // Update local struct
            existing_class.class_teacher_id = Some(tea_id);
            existing_class.updated_at = Utc::now();

            // Build update document
            let mut update_doc = Document::new();
            update_doc.insert(
                "class_teacher_id",
                bson::to_bson(&existing_class.class_teacher_id).map_err(|e| AppError {
                    message: format!("Failed to serialize class_teacher_id: {}", e),
                })?,
            );

            update_doc.insert(
                "updated_at",
                bson::to_bson(&existing_class.updated_at).map_err(|e| AppError {
                    message: format!("Failed to serialize updated_at: {}", e),
                })?,
            );

            let update_doc = doc! { "$set": update_doc };

            self.collection
                .update_one(doc! { "_id": cls_id }, update_doc)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to update class teacher: {}", e),
                })?;

            Ok(existing_class)
        } else {
            Err(AppError {
                message: "Class not found to assign class teacher".to_string(),
            })
        }
    }

    /// Add a subclass to a main class
    pub async fn add_subclass(
        &self,
        main_class_id: &IdType,
        subclass: &Class,
    ) -> Result<Class, AppError> {
        let main_obj_id = parse_object_id(main_class_id)?;

        // Verify the main class exists and is actually a main class
        let main_class = self
            .find_by_id(main_class_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Main class not found".to_string(),
            })?;

        // Ensure the main class has the correct level type
        if main_class.level_type == Some(ClassLevelType::SubClass) {
            return Err(AppError {
                message: "Target class is not a main class".to_string(),
            });
        }

        // Create the subclass
        let mut subclass_to_insert = subclass.clone();
        subclass_to_insert.level_type = Some(ClassLevelType::SubClass);
        subclass_to_insert.parent_class_id = Some(main_obj_id);

        let inserted_subclass = self.insert_class(&subclass_to_insert).await?;
        let subclass_id = inserted_subclass.id.ok_or_else(|| AppError {
            message: "Failed to get inserted subclass ID".to_string(),
        })?;

        // Update the main class's subclass_ids array
        let update_doc = doc! {
            "$push": {
                "subclass_ids": subclass_id
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_one(doc! { "_id": main_obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update main class with subclass: {}", e),
            })?;

        Ok(inserted_subclass)
    }

    /// Remove a subclass from its main class
    pub async fn remove_subclass(&self, subclass_id: &IdType) -> Result<(), AppError> {
        let subclass_obj_id = parse_object_id(subclass_id)?;

        // Get the subclass to find its parent
        let subclass = self
            .find_by_id(subclass_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Subclass not found".to_string(),
            })?;

        // Verify it's actually a subclass
        if subclass.level_type != Some(ClassLevelType::SubClass) {
            return Err(AppError {
                message: "Class is not a subclass".to_string(),
            });
        }

        let parent_class_id = subclass.parent_class_id.ok_or_else(|| AppError {
            message: "Subclass has no parent class".to_string(),
        })?;

        // Remove from parent's subclass_ids
        let update_parent_doc = doc! {
            "$pull": {
                "subclass_ids": subclass_obj_id
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_one(doc! { "_id": parent_class_id }, update_parent_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to remove subclass from parent: {}", e),
            })?;

        // Delete the subclass
        self.delete_class(subclass_id).await
    }

    /// Get all subclasses of a main class
    pub async fn get_subclasses(&self, main_class_id: &IdType) -> Result<Vec<Class>, AppError> {
        let main_obj_id = parse_object_id(main_class_id)?;

        let mut cursor = self
            .collection
            .find(doc! {
                "parent_class_id": main_obj_id,
                "level_type": "SubClass"
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find subclasses: {}", e),
            })?;

        let mut subclasses = Vec::new();
        while let Some(class) = cursor.next().await {
            subclasses.push(class.map_err(|e| AppError {
                message: format!("Failed to process subclass: {}", e),
            })?);
        }
        Ok(subclasses)
    }

    /// Get subclasses with full details (including school, teacher, etc.)
    pub async fn get_subclasses_with_others(
        &self,
        main_class_id: &IdType,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        let main_obj_id = parse_object_id(main_class_id)?;

        self.find_class_with_others(
            Some(doc! {
                "parent_class_id": main_obj_id,
                "level_type": "SubClass"
            }),
            None,
            None,
            None,
        )
        .await
    }

    /// Get the main class of a subclass
    pub async fn get_parent_class(&self, subclass_id: &IdType) -> Result<Option<Class>, AppError> {
        let subclass = self
            .find_by_id(subclass_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Subclass not found".to_string(),
            })?;

        if let Some(parent_id) = subclass.parent_class_id {
            self.find_by_id(&IdType::from_object_id(parent_id)).await
        } else {
            Ok(None)
        }
    }

    /// Move a subclass to a different main class
    pub async fn move_subclass(
        &self,
        subclass_id: &IdType,
        new_main_class_id: &IdType,
    ) -> Result<Class, AppError> {
        let subclass_obj_id = parse_object_id(subclass_id)?;
        let new_main_obj_id = parse_object_id(new_main_class_id)?;

        // Verify the new main class exists and is a main class
        let new_main_class = self
            .find_by_id(new_main_class_id)
            .await?
            .ok_or_else(|| AppError {
                message: "New main class not found".to_string(),
            })?;

        if new_main_class.level_type == Some(ClassLevelType::SubClass) {
            return Err(AppError {
                message: "Target class is not a main class".to_string(),
            });
        }

        // Get the subclass to find current parent
        let subclass = self
            .find_by_id(subclass_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Subclass not found".to_string(),
            })?;

        let current_parent_id = subclass.parent_class_id.ok_or_else(|| AppError {
            message: "Subclass has no parent class".to_string(),
        })?;

        // Start a transaction (if using MongoDB 4.0+ with transactions enabled)
        // For simplicity, we'll do multiple operations

        // Remove from old parent
        let remove_from_old = doc! {
            "$pull": {
                "subclass_ids": subclass_obj_id
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        // Add to new parent
        let add_to_new = doc! {
            "$push": {
                "subclass_ids": subclass_obj_id
            },
            "$set": {
                "updated_at":bson::to_bson(&Utc::now()).unwrap()
            }
        };

        // Update subclass parent reference
        let update_subclass = doc! {
            "$set": {
                "parent_class_id": new_main_obj_id,
                "updated_at":bson::to_bson(&Utc::now()).unwrap()
            }
        };

        // Execute updates
        self.collection
            .update_one(doc! { "_id": current_parent_id }, remove_from_old)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to remove from old parent: {}", e),
            })?;

        self.collection
            .update_one(doc! { "_id": new_main_obj_id }, add_to_new)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to add to new parent: {}", e),
            })?;

        self.collection
            .update_one(doc! { "_id": subclass_obj_id }, update_subclass)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update subclass: {}", e),
            })?;

        // Return updated subclass
        self.find_by_id(subclass_id).await?.ok_or_else(|| AppError {
            message: "Subclass not found after update".to_string(),
        })
    }

    /// Check if a class is a main class with subclasses
    pub async fn is_main_class_with_subclasses(&self, class_id: &IdType) -> Result<bool, AppError> {
        let class = self.find_by_id(class_id).await?.ok_or_else(|| AppError {
            message: "Class not found".to_string(),
        })?;

        Ok(class.level_type == Some(ClassLevelType::MainClass)
            && class.subclass_ids.as_ref().is_some_and(|v| !v.is_empty()))
    }

    /// Get all main classes (classes without parent_class_id and with MainClass level type)
    pub async fn get_main_classes(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<PaginatedClasses, AppError> {
        // 👇 Define extra MongoDB match for main classes
        let extra_match = Some(doc! {
            "level_type": "MainClass",
            "parent_class_id": { "$exists": false }
        });

        let paginated = self
            .get_all_classes(filter, limit, skip, extra_match)
            .await
            .map_err(|e| AppError { message: e.message })?;

        Ok(paginated)
    }

    /// Bulk add multiple subclasses to a main class
    pub async fn add_multiple_subclasses(
        &self,
        main_class_id: &IdType,
        subclasses: Vec<Class>,
    ) -> Result<Vec<Class>, AppError> {
        let main_obj_id = parse_object_id(main_class_id)?;

        // Verify main class exists
        let main_class = self
            .find_by_id(main_class_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Main class not found".to_string(),
            })?;

        if main_class.level_type == Some(ClassLevelType::SubClass) {
            return Err(AppError {
                message: "Target class is not a main class".to_string(),
            });
        }

        // Prepare subclasses for insertion
        let mut subclasses_to_insert = Vec::new();
        for mut subclass in subclasses {
            subclass.level_type = Some(ClassLevelType::SubClass);
            subclass.parent_class_id = Some(main_obj_id);
            subclasses_to_insert.push(subclass);
        }

        // Insert all subclasses
        let inserted_subclasses = self.create_many_classes(subclasses_to_insert).await?;
        let subclass_ids: Vec<ObjectId> = inserted_subclasses.iter().filter_map(|c| c.id).collect();

        if subclass_ids.is_empty() {
            return Ok(inserted_subclasses);
        }

        // --- FIX START ---
        // The previous error indicated that 'subclass_ids' can be 'null' in the database,
        // which causes '$push' to fail. We must use an update pipeline to
        // atomically check for null, set to an empty array if it is, and then
        // concatenate our new subclass IDs.

        // Convert our new subclass IDs into a BSON array for the pipeline
        let subclass_ids_bson = bson::to_bson(&subclass_ids).map_err(|e| AppError {
            message: format!("Failed to serialize subclass IDs for update: {}", e),
        })?;

        // Define the update pipeline
        let update_pipeline = vec![
            doc! {
                "$set": {
                    // Stage 1: Fix the 'subclass_ids' field.
                    // If it's null, set it to an empty array [].
                    // Otherwise, keep its existing value.
                    // Also set the 'updated_at' timestamp.
                    "subclass_ids": { "$ifNull": [ "$subclass_ids", [] ] },
                    "updated_at": bson::to_bson(&Utc::now()).unwrap()
                }
            },
            doc! {
                "$set": {
                    // Stage 2: Concatenate the (now guaranteed to be an array)
                    // 'subclass_ids' field with our new IDs.
                    "subclass_ids": {
                        "$concatArrays": [ "$subclass_ids", subclass_ids_bson ]
                    }
                }
            },
        ];
        // --- FIX END ---

        self.collection
            // Pass the update_pipeline (a Vec<Document>) instead of a single update_doc
            .update_one(doc! { "_id": main_obj_id }, update_pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update main class with subclasses: {}", e),
            })?;

        Ok(inserted_subclasses)
    }
}
