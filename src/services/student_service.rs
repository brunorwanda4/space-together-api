use mongodb::{
    bson::{self, doc, Document},
    Collection, Database,
};

use crate::{
    config::state::AppState,
    domain::{
        common_details::Paginated,
        join_school_request::{CreateJoinSchoolRequest, JoinRole},
        student::{Student, StudentPartial, StudentStatus, StudentWithRelations},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::student_pipeline::student_pipeline,
    repositories::base_repo::BaseRepository,
    services::{
        cloudinary_service::CloudinaryService,
        join_school_request_service::JoinSchoolRequestService,
    },
    utils::{
        email::is_valid_email,
        mongo_utils::{build_search_filter, extract_valid_fields},
        names::is_valid_name,
    },
};

pub struct StudentService {
    pub collection: Collection<Student>,
}

impl StudentService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Student>("students"),
        }
    }
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("email", true),
            IndexDef::single_with_partial(
                "user_id",
                true,
                doc! { "user_id": { "$type": "objectId" } },
                Some("user_id_objectid_unique"),
            ),
            IndexDef::single("school_id", false),
            IndexDef::single("class_id", false),
            IndexDef::single("creator_id", false),
            IndexDef::single("status", false),
            IndexDef::single("is_active", false),
            IndexDef::single_with_partial(
                "registration_number",
                true,
                doc! { "registration_number": { "$exists": true } },
                Some("registration_number_unique"),
            ),
            IndexDef::compound(vec![("school_id", 1), ("class_id", 1)], false),
            IndexDef::compound(vec![("school_id", 1), ("status", 1)], false),
        ];

        let repo = BaseRepository::new(
            self.collection
                .clone()
                .clone_with_type::<mongodb::bson::Document>(),
        );

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }
    // =========================
    // CREATE
    // =========================
    pub async fn create(
        &self,
        dto: Student,
        state: Option<&AppState>,
    ) -> Result<Student, AppError> {
        self.ensure_indexes().await?;
        if let Err(e) = is_valid_name(&dto.name) {
            return Err(AppError { message: e });
        }

        if let Err(e) = is_valid_email(&dto.email) {
            return Err(AppError { message: e });
        }

        if let Ok(student) = self.find_one(None, Some(doc! {"email": &dto.email})).await {
            return Err(AppError {
                message: format!("Email already exists: {}", student.email),
            });
        }

        if let Some(reg_num) = &dto.registration_number {
            if let Ok(student) = self
                .find_one(None, Some(doc! {"registration_number": reg_num}))
                .await
            {
                return Err(AppError {
                    message: format!(
                        "Registration number already exists: {:#?}",
                        student.registration_number
                    ),
                });
            }
        }

        // Image logic
        let mut partial = dto;
        if let Some(image_data) = partial.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;
            partial.image_id = Some(cloud_res.public_id);
            partial.image = Some(cloud_res.secure_url);
        }

        if matches!(partial.status, StudentStatus::Active) {
            partial.status = StudentStatus::Active;
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let student = repo
            .create::<Student>(extract_valid_fields(partial.to_document()?), None)
            .await?;

        if let Some(school_id) = student.school_id {
            if let Some(sent_by) = student.creator_id {
                if let Some(app_state) = state {
                    let create_request = CreateJoinSchoolRequest {
                        school_id: school_id.to_string(),
                        class_id: student.class_id.map(|id| id.to_hex()),
                        message: Some("Join School request".to_string()),
                        r#type: "Student".to_string(),
                        role: JoinRole::Student,
                        email: student.email.clone(),
                        sent_by: sent_by.clone().to_hex(),
                    };

                    let join_request_service =
                        JoinSchoolRequestService::new(&app_state.db.main_db());

                    let _join_request = join_request_service
                        .create(create_request, sent_by, app_state)
                        .await
                        .ok();
                }
            }
        }

        Ok(student)
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Student, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Student>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Student not found".into(),
            })
    }

    // =========================
    // GET ALL (PLAIN)
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Student>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "email",
            "registration_number",
            "_id",
            "user_id",
            "school_id",
            "class_id",
            "subclass_id",
            "phone",
            "gender",
            "admission_year",
            "status",
            "tags",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Student>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // =========================
    // UPDATE
    // =========================
    pub async fn update(&self, id: &IdType, update: &StudentPartial) -> Result<Student, AppError> {
        if let Some(ref name) = update.name {
            if let Err(e) = is_valid_name(name) {
                return Err(AppError { message: e });
            }
        }

        if let Some(ref email) = update.email {
            if let Err(e) = is_valid_email(email) {
                return Err(AppError { message: e });
            }
        }

        let existing_student = self.find_one(Some(id), None).await?;

        if let Some(ref email) = update.email {
            if existing_student.email != *email {
                if let Ok(student) = self.find_one(None, Some(doc! { "email": email })).await {
                    return Err(AppError {
                        message: format!("Email already exists: {}", student.email),
                    });
                }
            }
        }

        if let Some(reg_number) = update.registration_number.clone().flatten() {
            if existing_student.registration_number.as_ref() != Some(&reg_number) {
                if let Ok(student) = self
                    .find_one(None, Some(doc! { "registration_number": &reg_number }))
                    .await
                {
                    return Err(AppError {
                        message: format!(
                            "Registration number already exists: {:?}",
                            student.registration_number
                        ),
                    });
                }
            }
        }

        let mut update_data = update.clone();

        if let Some(new_image_data) = update.image.clone().flatten() {
            if Some(new_image_data.clone()) != existing_student.image {
                if let Some(old_image_id) = existing_student.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.image_id = Some(Some(cloud_res.public_id));
                update_data.image = Some(Some(cloud_res.secure_url));
            }
        }

        let full_doc = bson::to_document(&update_data).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Student>(id, extract_valid_fields(full_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Student, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let student = self.find_one(Some(id), None).await?;

        // Delete the student's image from Cloudinary if it exists
        if let Some(ref image_id) = student.image_id {
            CloudinaryService::delete_from_cloudinary(image_id)
                .await
                .ok();
        }

        repo.delete_one(id).await?;

        Ok(student)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<StudentWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            // =========================
            // BASE STRING + OBJECT ID SEARCH
            // =========================
            let search = build_search_filter(
                Some(f),
                &[
                    "name",
                    "email",
                    "registration_number",
                    "phone",
                    "gender",
                    "status",
                    "_id",
                    "user_id",
                    "school_id",
                    "class_id",
                    "subclass_id",
                    "tags",
                ],
            );

            match_stage.extend(search);
        }

        let pipeline = student_pipeline(match_stage);

        repo.aggregate_with_paginate::<StudentWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<StudentWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<StudentWithRelations>(student_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Student not found".into(),
            })
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count_students(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "email",
            "registration_number",
            "tags",
            "gender",
            "school_id",
            "class_id",
            "subclass_id",
            "national_id",
            "status",
        ];

        repo.count(filter, &searchable, extra_match).await
    }
}
