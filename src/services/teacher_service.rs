use chrono::Utc;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    config::state::AppState,
    domain::{
        common_details::Paginated,
        join_school_request::{CreateJoinSchoolRequest, JoinRole},
        teacher::{Teacher, UpdateTeacher},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::base_repo::BaseRepository,
    services::cloudinary_service::CloudinaryService,
    utils::{
        email::is_valid_email,
        join_school_request_controller_utils::create_join_school_request_controller,
        mongo_utils::extract_valid_fields, names::is_valid_name,
    },
};

pub struct TeacherService {
    pub collection: Collection<Teacher>,
}

impl TeacherService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Teacher>("teachers"),
        }
    }

    // =========================
    // INDEXES
    // =========================
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
            IndexDef::single("creator_id", false),
            IndexDef::single("type", false),
            IndexDef::single("is_active", false),
            IndexDef::compound(vec![("school_id", 1), ("type", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(
        &self,
        mut dto: Teacher,
        state: Option<&AppState>,
    ) -> Result<Teacher, AppError> {
        self.ensure_indexes().await?;

        // Validation
        if let Err(e) = is_valid_name(&dto.name) {
            return Err(AppError { message: e });
        }

        if let Err(e) = is_valid_email(&dto.email) {
            return Err(AppError { message: e });
        }

        // Unique email
        if let Ok(existing) = self
            .find_one(None, Some(doc! { "email": &dto.email }))
            .await
        {
            return Err(AppError {
                message: format!("Email already exists: {}", existing.email),
            });
        }

        // Image handling
        if let Some(image_data) = dto.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;

            dto.image_id = Some(cloud_res.public_id);
            dto.image = Some(cloud_res.secure_url);
        }

        let full_doc = bson::to_document(&dto).map_err(|e| AppError {
            message: format!("Failed to serialize teacher: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let teacher = repo
            .create::<Teacher>(extract_valid_fields(full_doc), None)
            .await?;

        if let Some(school_id) = teacher.school_id {
            if let Some(sent_by) = teacher.creator_id {
                if let Some(app_state) = state {
                    let create_request = CreateJoinSchoolRequest {
                        school_id: school_id.to_string(),
                        class_id: None,
                        message: Some("Join School request".to_string()),
                        r#type: "Teacher".to_string(),
                        role: JoinRole::Teacher,
                        email: teacher.email.clone(),
                        sent_by: sent_by.clone().to_hex(),
                    };

                    let join_request_controller = create_join_school_request_controller(app_state);

                    let _join_request = join_request_controller
                        .create_join_request(create_request, sent_by)
                        .await
                        .ok();
                }
            }
        }

        Ok(teacher)
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Teacher, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Teacher>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Teacher not found".into(),
            })
    }

    // =========================
    // GET ALL
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Teacher>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "email", "phone", "user_id", "_id", "tags"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Teacher>(filter, &searchable, limit, skip, extra_match)
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
    pub async fn update(&self, id: &IdType, update: &UpdateTeacher) -> Result<Teacher, AppError> {
        // Validation
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

        let existing = self.find_one(Some(id), None).await?;

        // Unique email check
        if let Some(ref email) = update.email {
            if existing.email != *email {
                if let Ok(t) = self.find_one(None, Some(doc! { "email": email })).await {
                    return Err(AppError {
                        message: format!("Email already exists: {}", t.email),
                    });
                }
            }
        }

        let mut update_data = update.clone();

        // Image update
        if let Some(new_image) = update.image.clone().flatten() {
            if Some(new_image.clone()) != existing.image {
                if let Some(old_image_id) = existing.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image)
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
        repo.update_one_and_fetch::<Teacher>(id, extract_valid_fields(full_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Teacher, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let teacher = self.find_one(Some(id), None).await?;

        if let Some(ref image_id) = teacher.image_id {
            CloudinaryService::delete_from_cloudinary(image_id)
                .await
                .ok();
        }

        repo.delete_one(id).await?;

        Ok(teacher)
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count_teachers(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "email",
            "phone",
            "tags",
            "school_id",
            "type",
            "is_active",
        ];

        repo.count(filter, &searchable, extra_match).await
    }

    pub async fn add_subjects_to_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = IdType::to_object_id(teacher_id)?;
        let update = self
            .collection
            .find_one_and_update(
                doc! {"_id": obj_id},
                doc! {
                    "$addToSet": {
                        "subject_ids": { "$each": &subject_ids }
                    },
                    "$set": {
                        "updated_at": bson::to_bson(&Utc::now()).unwrap()
                    }
                },
            )
            .await?;

        update.ok_or(AppError {
            message: "Teacher not found".into(),
        })
    }

    pub async fn add_classes_to_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = IdType::to_object_id(teacher_id)?;
        let update = self
            .collection
            .find_one_and_update(
                doc! {"_id": obj_id},
                doc! {
                    "$addToSet": {
                        "class_ids": {
                            "$each": &class_ids
                        }
                    },
                    "$set": {
                        "updated_at": bson::to_bson(&Utc::now()).unwrap()
                    }
                },
            )
            .await?;

        update.ok_or(AppError {
            message: "Teacher not found".into(),
        })
    }

    pub async fn remove_classes_from_teacher(
        &self,
        teacher_id: &IdType,
        class_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = IdType::to_object_id(teacher_id)?;
        let update = self
            .collection
            .find_one_and_update(
                doc! {"_id": obj_id},
                doc! {
                    "$pullAll": {
                        "class_ids": &class_ids
                    },
                    "$set": {
                        "updated_at": bson::to_bson(&Utc::now()).unwrap()
                    }
                },
            )
            .await?;

        update.ok_or(AppError {
            message: "Teacher not found".into(),
        })
    }

    pub async fn remove_subjects_from_teacher(
        &self,
        teacher_id: &IdType,
        subject_ids: Vec<ObjectId>,
    ) -> Result<Teacher, AppError> {
        let obj_id = IdType::to_object_id(teacher_id)?;
        let update = self
            .collection
            .find_one_and_update(
                doc! {"_id": obj_id},
                doc! {
                    "$pullAll": {
                        "subject_ids": &subject_ids
                    },
                    "$set": {
                        "updated_at": bson::to_bson(&Utc::now()).unwrap()
                    }
                },
            )
            .await?;

        update.ok_or(AppError {
            message: "Teacher not found".into(),
        })
    }
}
