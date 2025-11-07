use crate::domain::common_details::Gender;
use crate::domain::student::{
    BulkStudentIds, BulkStudentTags, BulkUpdateStudentStatus, Student, StudentStatus,
    StudentWithRelations, UpdateStudent,
};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::helpers::repo_helpers::safe_create_index;
use crate::models::id_model::IdType;
use crate::pipeline::student_pipeline::student_with_relations_pipeline;
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct StudentRepo {
    pub collection: Collection<Student>,
}

impl StudentRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Student>("students"),
        }
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<StudentWithRelations>, AppError> {
        let mut pipeline = vec![];

        // 🔍 Add search/filter functionality
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"  // case-insensitive
            };

            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "email": &regex },
                        { "registration_number": &regex },
                        { "tags": &regex },
                    ]
                }
            });
        }

        // 🧩 Merge with the relations pipeline
        let mut relations_pipeline = student_with_relations_pipeline(doc! {});
        pipeline.append(&mut relations_pipeline);

        // 🕒 Add sorting by updated_at (most recent first)
        pipeline.insert(0, doc! { "$sort": { "updated_at": -1 } });

        // ⏭️ Pagination (skip, limit)
        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }

        if let Some(l) = limit {
            pipeline.push(doc! { "$limit": l });
        } else {
            pipeline.push(doc! { "$limit": 50 }); // default limit
        }

        // 🧠 Perform aggregation
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
    ) -> Result<Option<StudentWithRelations>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            student_with_relations_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<Student>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find student by id: {}", e),
            })
    }

    pub async fn find_by_user_id(&self, user_id: &IdType) -> Result<Option<Student>, AppError> {
        let obj_id = parse_object_id(user_id)?;
        self.collection
            .find_one(doc! { "user_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find student by user_id: {}", e),
            })
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Student>, AppError> {
        self.collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find student by email: {}", e),
            })
    }

    pub async fn find_by_school_id(&self, school_id: &IdType) -> Result<Vec<Student>, AppError> {
        let obj_id = parse_object_id(school_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "school_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find students by school_id: {}", e),
            })?;

        let mut students = Vec::new();
        while let Some(student) = cursor.next().await {
            students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }
        Ok(students)
    }

    pub async fn find_by_class_id(&self, class_id: &IdType) -> Result<Vec<Student>, AppError> {
        let obj_id = parse_object_id(class_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "class_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find students by class_id: {}", e),
            })?;

        let mut students = Vec::new();
        while let Some(student) = cursor.next().await {
            students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }
        Ok(students)
    }

    pub async fn find_by_creator_id(&self, creator_id: &IdType) -> Result<Vec<Student>, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find students by creator_id: {}", e),
            })?;

        let mut students = Vec::new();
        while let Some(student) = cursor.next().await {
            students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }
        Ok(students)
    }

    pub async fn find_by_status(&self, status: StudentStatus) -> Result<Vec<Student>, AppError> {
        let mut cursor = self
            .collection
            .find(
                doc! { "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize student status: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find students by status: {}", e),
            })?;

        let mut students = Vec::new();
        while let Some(student) = cursor.next().await {
            students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }
        Ok(students)
    }

    pub async fn find_by_registration_number(
        &self,
        registration_number: &str,
    ) -> Result<Option<Student>, AppError> {
        self.collection
            .find_one(doc! { "registration_number": registration_number })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find student by registration_number: {}", e),
            })
    }

    pub async fn insert_student(&self, student: &Student) -> Result<Student, AppError> {
        self.ensure_indexes().await?;

        let mut to_insert = student.clone();
        to_insert.id = None;
        to_insert.created_at = Utc::now();
        to_insert.updated_at = Utc::now();

        let res = self
            .collection
            .insert_one(&to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert student: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted student id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Student not found after insert".to_string(),
            })
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        // === Define indexes ===
        let email_index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let user_id_index = IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    // ✅ safer than $ne:null — allows only valid ObjectId types
                    .partial_filter_expression(doc! { "user_id": { "$type": "objectId" } })
                    .build(),
            )
            .build();

        let school_index = IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let class_index = IndexModel::builder()
            .keys(doc! { "class_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let creator_index = IndexModel::builder()
            .keys(doc! { "creator_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let is_active_index = IndexModel::builder()
            .keys(doc! { "is_active": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let registration_number_index = IndexModel::builder()
            .keys(doc! { "registration_number": 1 })
            .options(IndexOptions::builder().unique(true).sparse(true).build())
            .build();

        let school_class_index = IndexModel::builder()
            .keys(doc! { "school_id": 1, "class_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let school_status_index = IndexModel::builder()
            .keys(doc! { "school_id": 1, "status": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        // === Use Document collection for safety ===
        let doc_collection = self.collection.clone_with_type::<Document>();

        // === Create all indexes safely ===
        safe_create_index(&doc_collection, email_index, "email").await?;
        safe_create_index(&doc_collection, user_id_index, "user_id").await?;
        safe_create_index(&doc_collection, school_index, "school_id").await?;
        safe_create_index(&doc_collection, class_index, "class_id").await?;
        safe_create_index(&doc_collection, creator_index, "creator_id").await?;
        safe_create_index(&doc_collection, status_index, "status").await?;
        safe_create_index(&doc_collection, is_active_index, "is_active").await?;
        safe_create_index(
            &doc_collection,
            registration_number_index,
            "registration_number",
        )
        .await?;
        safe_create_index(&doc_collection, school_class_index, "school_id+class_id").await?;
        safe_create_index(&doc_collection, school_status_index, "school_id+status").await?;

        Ok(())
    }

    pub async fn get_all_students(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Student>, AppError> {
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
                        { "email": &regex },
                        { "registration_number": &regex },
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
                message: format!("Failed to fetch students: {}", e),
            })?;

        let mut students = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate students: {}", e),
        })? {
            let student: Student = mongodb::bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize student: {}", e),
            })?;
            students.push(student);
        }

        Ok(students)
    }

    pub async fn get_active_students(&self) -> Result<Vec<Student>, AppError> {
        let mut cursor = self
            .collection
            .find(doc! { "is_active": true })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find active students: {}", e),
            })?;

        let mut students = Vec::new();
        while let Some(student) = cursor.next().await {
            students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }
        Ok(students)
    }

    pub async fn update_student(
        &self,
        id: &IdType,
        update: &UpdateStudent,
    ) -> Result<Student, AppError> {
        let obj_id = parse_object_id(id)?;

        // Create update document manually to handle Option fields
        let mut update_doc = Document::new();

        if let Some(name) = &update.name {
            update_doc.insert("name", name);
        }
        if let Some(email) = &update.email {
            update_doc.insert("email", email);
        }
        if let Some(phone) = &update.phone {
            update_doc.insert("phone", phone);
        }
        if let Some(image) = &update.image {
            update_doc.insert("image", image);
        }
        if let Some(image_id) = &update.image_id {
            update_doc.insert("image_id", image_id);
        }
        if let Some(gender) = &update.gender {
            update_doc.insert("gender", gender.to_string());
        }
        if let Some(user_id) = &update.user_id {
            update_doc.insert("user_id", user_id.to_string());
        }

        if let Some(date_of_birth) = &update.date_of_birth {
            update_doc.insert(
                "date_of_birth",
                doc! {
                    "year": date_of_birth.year,
                    "month": date_of_birth.month,
                    "day": date_of_birth.day
                },
            );
        }
        if let Some(registration_number) = &update.registration_number {
            update_doc.insert("registration_number", registration_number);
        }
        if let Some(admission_year) = update.admission_year {
            update_doc.insert("admission_year", admission_year);
        }
        if let Some(status) = &update.status {
            update_doc.insert(
                "status",
                bson::to_bson(status).map_err(|e| AppError {
                    message: format!("Failed to serialize student status: {}", e),
                })?,
            );
        }
        if let Some(is_active) = update.is_active {
            update_doc.insert("is_active", is_active);
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
                message: format!("Failed to update student: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Student not found after update".to_string(),
        })
    }

    pub async fn delete_student(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete student: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No student deleted; it may not exist".to_string(),
            });
        }
        Ok(())
    }

    pub async fn count_by_school_id(
        &self,
        school_id: &IdType,
        gender: Option<Gender>,
        status: Option<StudentStatus>,
    ) -> Result<u64, AppError> {
        let obj_id = parse_object_id(school_id)?;

        // Base filter
        let mut filter = doc! { "school_id": obj_id };

        // Add optional filters
        if let Some(g) = gender {
            filter.insert("gender", g.to_string());
        }

        if let Some(s) = status {
            filter.insert("status", s.to_string());
        }

        self.collection
            .count_documents(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count students by school_id: {}", e),
            })
    }

    pub async fn count_by_class_id(&self, class_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(class_id)?;
        self.collection
            .count_documents(doc! { "class_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count students by class_id: {}", e),
            })
    }

    pub async fn count_by_creator_id(&self, creator_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        self.collection
            .count_documents(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count students by creator_id: {}", e),
            })
    }

    pub async fn count_by_status(&self, status: StudentStatus) -> Result<u64, AppError> {
        self.collection
            .count_documents(
                doc! { "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize student status: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count students by status: {}", e),
            })
    }

    // Bulk operations
    pub async fn create_many_students(
        &self,
        students: Vec<Student>,
    ) -> Result<Vec<Student>, AppError> {
        self.ensure_indexes().await?;

        let mut students_to_insert = Vec::with_capacity(students.len());
        let now = Utc::now();

        for mut student in students {
            student.id = None;
            student.created_at = now;
            student.updated_at = now;
            students_to_insert.push(student);
        }

        let result = self
            .collection
            .insert_many(&students_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert multiple students: {}", e),
            })?;

        let inserted_ids: Vec<ObjectId> = result
            .inserted_ids
            .values()
            .filter_map(|bson| bson.as_object_id())
            .collect();

        if inserted_ids.len() != students_to_insert.len() {
            return Err(AppError {
                message: "Failed to get all inserted student IDs".to_string(),
            });
        }

        let mut inserted_students = Vec::with_capacity(inserted_ids.len());
        for id in inserted_ids {
            let student = self.find_by_id(&IdType::from_object_id(id)).await?;
            if let Some(student) = student {
                inserted_students.push(student);
            }
        }

        Ok(inserted_students)
    }

    pub async fn update_many_students(
        &self,
        updates: Vec<(IdType, UpdateStudent)>,
    ) -> Result<Vec<Student>, AppError> {
        let mut updated_students = Vec::with_capacity(updates.len());

        for (id, update) in updates {
            match self.update_student(&id, &update).await {
                Ok(student) => updated_students.push(student),
                Err(e) => {
                    eprintln!("Failed to update student {:?}: {}", id, e.message);
                }
            }
        }

        if updated_students.is_empty() {
            return Err(AppError {
                message: "No students were successfully updated".to_string(),
            });
        }

        Ok(updated_students)
    }

    pub async fn bulk_update_status(
        &self,
        request: &BulkUpdateStudentStatus,
    ) -> Result<Vec<Student>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$set": {
                "status": bson::to_bson(&request.status).map_err(|e| AppError {
                    message: format!("Failed to serialize student status: {}", e),
                })?,
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk update status: {}", e),
            })?;

        // Return updated students
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated students: {}", e),
            })?;

        let mut updated_students = Vec::new();
        while let Some(student) = cursor.next().await {
            updated_students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }

        Ok(updated_students)
    }

    pub async fn bulk_add_tags(&self, request: &BulkStudentTags) -> Result<Vec<Student>, AppError> {
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

        // Return updated students
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated students: {}", e),
            })?;

        let mut updated_students = Vec::new();
        while let Some(student) = cursor.next().await {
            updated_students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }

        Ok(updated_students)
    }

    pub async fn bulk_remove_tags(
        &self,
        request: &BulkStudentTags,
    ) -> Result<Vec<Student>, AppError> {
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

        // Return updated students
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": &object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated students: {}", e),
            })?;

        let mut updated_students = Vec::new();
        while let Some(student) = cursor.next().await {
            updated_students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }

        Ok(updated_students)
    }

    pub async fn delete_many_students(&self, request: &BulkStudentIds) -> Result<u64, AppError> {
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
                message: format!("Failed to delete multiple students: {}", e),
            })?;

        Ok(result.deleted_count)
    }

    pub async fn transfer_students_to_class(
        &self,
        student_ids: &BulkStudentIds,
        new_class_id: &IdType,
    ) -> Result<Vec<Student>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = student_ids
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;
        let new_class_obj_id = parse_object_id(new_class_id)?;

        let update_doc = doc! {
            "$set": {
                "class_id": new_class_obj_id,
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to transfer students to class: {}", e),
            })?;

        // Return updated students
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch transferred students: {}", e),
            })?;

        let mut updated_students = Vec::new();
        while let Some(student) = cursor.next().await {
            updated_students.push(student.map_err(|e| AppError {
                message: format!("Failed to process student: {}", e),
            })?);
        }

        Ok(updated_students)
    }
}
