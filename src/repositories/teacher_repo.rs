use crate::domain::common_details::Gender;
use crate::domain::teacher::{
    BulkTeacherIds, BulkTeacherTags, BulkUpdateTeacherActive, PaginatedTeachers,
    PrepareTeacherRequest, Teacher, TeacherType, TeacherWithRelations, UpdateTeacher,
};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::models::mongo_model::IndexDef;
use crate::pipeline::teacher_pipeline::teacher_with_relations_pipeline;
use crate::repositories::base_repo::BaseRepository;
use crate::utils::mongo_utils::extract_valid_fields;
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

pub struct TeacherRepo {
    pub collection: Collection<Teacher>,
}

impl TeacherRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Teacher>("teachers"),
        }
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        let mut pipeline = vec![];

        // 🔍 Add search/filter functionality
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"  // case-insensitive search
            };

            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "email": &regex },
                        { "phone": &regex },
                        { "tags": &regex },
                    ]
                }
            });
        }

        // 🧩 Merge with teacher relations pipeline
        let mut relations_pipeline = teacher_with_relations_pipeline(doc! {});
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
    ) -> Result<Option<TeacherWithRelations>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            teacher_with_relations_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<Teacher>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teacher by id: {}", e),
            })
    }

    pub async fn find_by_user_id(&self, user_id: &IdType) -> Result<Option<Teacher>, AppError> {
        let obj_id = parse_object_id(user_id)?;
        self.collection
            .find_one(doc! { "user_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teacher by user_id: {}", e),
            })
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Teacher>, AppError> {
        self.collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teacher by email: {}", e),
            })
    }

    pub async fn find_by_school_id(&self, school_id: &IdType) -> Result<Vec<Teacher>, AppError> {
        let obj_id = parse_object_id(school_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "school_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teachers by school_id: {}", e),
            })?;

        let mut teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }
        Ok(teachers)
    }

    pub async fn find_by_class_id(
        &self,
        class_id: &IdType,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Teacher>, AppError> {
        let obj_id = parse_object_id(class_id)?;

        // If limit or skip is explicitly provided, use paginated method
        // Otherwise, return all records to maintain backward compatibility
        if limit.is_some() || skip.is_some() {
            let paginated_teachers = self
                .get_all_teachers(filter, limit, skip, Some(doc! { "class_ids": obj_id }))
                .await?;
            Ok(paginated_teachers.teachers)
        } else {
            // Return all records when no pagination is requested (original behavior)
            // Support filter without pagination by using a direct query
            let match_doc = if let Some(f) = filter {
                let regex = doc! {
                    "$regex": f,
                    "$options": "i"
                };
                doc! {
                    "$and": [
                        { "class_ids": obj_id },
                        {
                            "$or": [
                                { "name": &regex },
                                { "email": &regex },
                                { "phone": &regex },
                                { "tags": &regex },
                            ]
                        }
                    ]
                }
            } else {
                doc! { "class_ids": obj_id }
            };

            let mut cursor = self
                .collection
                .find(match_doc)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to find teachers by class_id: {}", e),
                })?;

            let mut teachers = Vec::new();
            while let Some(teacher) = cursor.next().await {
                teachers.push(teacher.map_err(|e| AppError {
                    message: format!("Failed to process teacher: {}", e),
                })?);
            }
            Ok(teachers)
        }
    }

    pub async fn find_by_subject_id(&self, subject_id: &IdType) -> Result<Vec<Teacher>, AppError> {
        let obj_id = parse_object_id(subject_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "subject_ids": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teachers by subject_id: {}", e),
            })?;

        let mut teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }
        Ok(teachers)
    }

    pub async fn find_by_creator_id(&self, creator_id: &IdType) -> Result<Vec<Teacher>, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teachers by creator_id: {}", e),
            })?;

        let mut teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }
        Ok(teachers)
    }

    pub async fn find_by_type(&self, teacher_type: TeacherType) -> Result<Vec<Teacher>, AppError> {
        let mut cursor = self
            .collection
            .find(
                doc! { "type": bson::to_bson(&teacher_type).map_err(|e| AppError {
                    message: format!("Failed to serialize teacher type: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teachers by type: {}", e),
            })?;

        let mut teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }
        Ok(teachers)
    }

    pub async fn find_by_school_and_type(
        &self,
        school_id: &IdType,
        teacher_type: TeacherType,
    ) -> Result<Vec<Teacher>, AppError> {
        let school_obj_id = parse_object_id(school_id)?;
        let mut cursor = self
            .collection
            .find(doc! {
                "school_id": school_obj_id,
                "type": bson::to_bson(&teacher_type).map_err(|e| AppError {
                    message: format!("Failed to serialize teacher type: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find teachers by school_id and type: {}", e),
            })?;

        let mut teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }
        Ok(teachers)
    }

    pub async fn insert_teacher(&self, teacher: &Teacher) -> Result<Teacher, AppError> {
        self.ensure_indexes().await?;

        let mut to_insert = teacher.clone();
        to_insert.id = None;
        to_insert.created_at = Utc::now();
        to_insert.updated_at = Utc::now();

        let res = self
            .collection
            .insert_one(&to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert teacher: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted teacher id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Teacher not found after insert".to_string(),
            })
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("email", true),
            IndexDef::single("user_id", true),
            IndexDef::single("school_id", false),
            IndexDef::single("class_ids", false),
            IndexDef::single("subject_ids", false),
            IndexDef::single("creator_id", false),
            IndexDef::single("type", false),
            IndexDef::single("is_active", false),
            /* --------------------------------
               Compound indexes
            ----------------------------------*/
            // school + type
            IndexDef::compound(vec![("school_id", 1), ("type", 1)], false),
            // school + is_active
            IndexDef::compound(vec![("school_id", 1), ("is_active", 1)], false),
            // class_ids + subject_ids
            IndexDef::compound(vec![("class_ids", 1), ("subject_ids", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;

        Ok(())
    }

    pub async fn get_all_teachers(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_filter: Option<Document>, // Additional conditions like is_active
    ) -> Result<PaginatedTeachers, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable_fields = [
            "name",
            "email",
            "phone",
            "tags",
            "gender",
            "_id",
            "school_id",
        ];

        let (teachers, total, total_pages, current_page) = base_repo
            .get_all::<Teacher>(filter, &searchable_fields, limit, skip, extra_filter)
            .await?;
        Ok(PaginatedTeachers {
            teachers,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn get_active_teachers(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Teacher>, AppError> {
        let class = self
            .get_all_teachers(
                filter,                           // no search text
                limit,                            // no limit
                skip,                             // no skip
                Some(doc! { "is_active": true }), // extra filter
            )
            .await?;

        Ok(class.teachers)
    }

    pub async fn update_teacher(
        &self,
        id: &IdType,
        update: &UpdateTeacher,
    ) -> Result<Teacher, AppError> {
        let full_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        // Create update document manually to handle Option fields
        let update_doc = extract_valid_fields(full_doc);

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<Teacher>(id, update_doc).await
    }

    pub async fn delete_teacher(&self, id: &IdType) -> Result<(), AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.delete_one(id).await
    }

    pub async fn count_by_school_id(
        &self,
        school_id: &IdType,
        gender: Option<Gender>,
        teacher_type: Option<TeacherType>,
    ) -> Result<u64, AppError> {
        let obj_id = parse_object_id(school_id)?;

        // Base filter
        let mut filter = doc! { "school_id": obj_id };

        // Optional filters
        if let Some(g) = gender {
            filter.insert("gender", g.to_string());
        }

        if let Some(t) = teacher_type {
            filter.insert("type", t.to_string());
        }

        self.collection
            .count_documents(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count teachers by school_id: {}", e),
            })
    }

    pub async fn count_by_class_id(&self, class_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(class_id)?;
        self.collection
            .count_documents(doc! { "class_ids": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count teachers by class_id: {}", e),
            })
    }

    pub async fn count_by_subject_id(&self, subject_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(subject_id)?;
        self.collection
            .count_documents(doc! { "subject_ids": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count teachers by subject_id: {}", e),
            })
    }

    pub async fn count_by_creator_id(&self, creator_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        self.collection
            .count_documents(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count teachers by creator_id: {}", e),
            })
    }

    pub async fn count_by_type(&self, teacher_type: TeacherType) -> Result<u64, AppError> {
        self.collection
            .count_documents(
                doc! { "type": bson::to_bson(&teacher_type).map_err(|e| AppError {
                    message: format!("Failed to serialize teacher type: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count teachers by type: {}", e),
            })
    }

    // Bulk operations
    pub async fn create_many_teachers(
        &self,
        teachers: Vec<Teacher>,
    ) -> Result<Vec<Teacher>, AppError> {
        self.ensure_indexes().await?;

        let mut teachers_to_insert = Vec::with_capacity(teachers.len());
        let now = Utc::now();

        for mut teacher in teachers {
            teacher.id = None;
            teacher.created_at = now;
            teacher.updated_at = now;
            teachers_to_insert.push(teacher);
        }

        let result = self
            .collection
            .insert_many(&teachers_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert multiple teachers: {}", e),
            })?;

        let inserted_ids: Vec<ObjectId> = result
            .inserted_ids
            .values()
            .filter_map(|bson| bson.as_object_id())
            .collect();

        if inserted_ids.len() != teachers_to_insert.len() {
            return Err(AppError {
                message: "Failed to get all inserted teacher IDs".to_string(),
            });
        }

        let mut inserted_teachers = Vec::with_capacity(inserted_ids.len());
        for id in inserted_ids {
            let teacher = self.find_by_id(&IdType::from_object_id(id)).await?;
            if let Some(teacher) = teacher {
                inserted_teachers.push(teacher);
            }
        }

        Ok(inserted_teachers)
    }

    pub async fn prepare_teachers(
        &self,
        request: &PrepareTeacherRequest,
    ) -> Result<Vec<Teacher>, AppError> {
        let mut prepared_teachers = Vec::new();

        for mut teacher in request.teachers.clone() {
            // Set school_id and creator_id from request if provided
            if let Some(school_id) = &request.school_id {
                teacher.school_id = Some(parse_object_id(&IdType::String(school_id.clone()))?);
            }
            if let Some(creator_id) = &request.creator_id {
                teacher.creator_id = Some(parse_object_id(&IdType::String(creator_id.clone()))?);
            }

            prepared_teachers.push(teacher);
        }

        Ok(prepared_teachers)
    }

    pub async fn update_many_teachers(
        &self,
        updates: Vec<(IdType, UpdateTeacher)>,
    ) -> Result<Vec<Teacher>, AppError> {
        let mut updated_teachers = Vec::with_capacity(updates.len());

        for (id, update) in updates {
            match self.update_teacher(&id, &update).await {
                Ok(teacher) => updated_teachers.push(teacher),
                Err(e) => {
                    eprintln!("Failed to update teacher {:?}: {}", id, e.message);
                }
            }
        }

        if updated_teachers.is_empty() {
            return Err(AppError {
                message: "No teachers were successfully updated".to_string(),
            });
        }

        Ok(updated_teachers)
    }

    pub async fn bulk_update_active(
        &self,
        request: &BulkUpdateTeacherActive,
    ) -> Result<Vec<Teacher>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$set": {
                "is_active": request.is_active,
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk update active status: {}", e),
            })?;

        // Return updated teachers
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated teachers: {}", e),
            })?;

        let mut updated_teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            updated_teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }

        Ok(updated_teachers)
    }

    pub async fn bulk_add_tags(&self, request: &BulkTeacherTags) -> Result<Vec<Teacher>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$addToSet": {
                "tags": { "$each": &request.tags }
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk add tags: {}", e),
            })?;

        // Return updated teachers
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated teachers: {}", e),
            })?;

        let mut updated_teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            updated_teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }

        Ok(updated_teachers)
    }

    pub async fn bulk_remove_tags(
        &self,
        request: &BulkTeacherTags,
    ) -> Result<Vec<Teacher>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$pullAll": {
                "tags": &request.tags
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk remove tags: {}", e),
            })?;

        // Return updated teachers
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": &object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated teachers: {}", e),
            })?;

        let mut updated_teachers = Vec::new();
        while let Some(teacher) = cursor.next().await {
            updated_teachers.push(teacher.map_err(|e| AppError {
                message: format!("Failed to process teacher: {}", e),
            })?);
        }

        Ok(updated_teachers)
    }

    pub async fn delete_many_teachers(&self, request: &BulkTeacherIds) -> Result<u64, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let result = self
            .collection
            .delete_many(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete multiple teachers: {}", e),
            })?;

        Ok(result.deleted_count)
    }

    pub async fn add_classes_to_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let teacher_obj_id = ObjectId::parse_str(teacher_id.as_string()).map_err(|e| AppError {
            message: format!("Invalid teacher id: {}", e),
        })?;

        let filter = doc! { "_id": &teacher_obj_id };

        // ✅ Step 1: Ensure `class_ids` is an array (initialize if missing or null)
        self.collection
            .update_one(
                doc! {
                    "_id": &teacher_obj_id,
                    "$or": [
                        { "class_ids": { "$exists": false } },
                        { "class_ids": bson::Bson::Null }
                    ]
                },
                doc! {
                    "$set": { "class_ids": bson::to_bson(&Vec::<ObjectId>::new()).unwrap() }
                },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to initialize class_ids field: {}", e),
            })?;

        // ✅ Step 2: Add new class IDs safely
        self.collection
            .update_one(
                filter.clone(),
                doc! {
                    "$addToSet": {
                        "class_ids": { "$each": &class_ids }
                    },
                    "$set": {
                        "updated_at": bson::to_bson(&Utc::now()).unwrap()
                    }
                },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to add classes to teacher: {}", e),
            })?;

        // ✅ Step 3: Return the updated teacher
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated teacher: {}", e),
            })?
            .ok_or(AppError {
                message: "Teacher not found after adding classes".to_string(),
            })
    }

    pub async fn add_subjects_to_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = parse_object_id(teacher_id)?;

        let update_doc = doc! {
            "$addToSet": {
                "subject_ids": { "$each": &subject_ids }
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to add subjects to teacher: {}", e),
            })?;

        self.find_by_id(teacher_id).await?.ok_or(AppError {
            message: "Teacher not found after adding subjects".to_string(),
        })
    }

    pub async fn remove_classes_from_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = parse_object_id(teacher_id)?;

        let update_doc = doc! {
            "$pullAll": {
                "class_ids": &class_ids
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to remove classes from teacher: {}", e),
            })?;

        self.find_by_id(teacher_id).await?.ok_or(AppError {
            message: "Teacher not found after removing classes".to_string(),
        })
    }

    pub async fn remove_subjects_from_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = parse_object_id(teacher_id)?;

        let update_doc = doc! {
            "$pullAll": {
                "subject_ids": &subject_ids
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to remove subjects from teacher: {}", e),
            })?;

        self.find_by_id(teacher_id).await?.ok_or(AppError {
            message: "Teacher not found after removing subjects".to_string(),
        })
    }
}
