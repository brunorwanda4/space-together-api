use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        class::{Class, ClassWithOthers, UpdateClass},
        common_details::{Image, Paginated},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::class_pipeline::class_pipeline,
    repositories::base_repo::BaseRepository,
    services::cloudinary_service::CloudinaryService,
    utils::{code::generate_code, mongo_utils::extract_valid_fields, names::is_valid_username},
};

pub struct ClassService {
    pub collection: Collection<Class>,
}

impl ClassService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Class>("classes"),
        }
    }

    // =========================
    // INDEXES
    // =========================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", false),
            IndexDef::single("code", true),
            IndexDef::single("username", true),
            IndexDef::single("school_id", false),
            IndexDef::single("creator_id", false),
            IndexDef::single("class_teacher_id", false),
            IndexDef::single("main_class_id", false),
            IndexDef::single("trade_id", false),
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
    pub async fn create(&self, mut dto: Class) -> Result<Class, AppError> {
        self.ensure_indexes().await?;

        is_valid_username(&dto.username).map_err(|e| AppError { message: e })?;

        // Ensure unique username
        if let Ok(_) = self
            .find_one(None, Some(doc! { "username": &dto.username }))
            .await
        {
            return Err(AppError {
                message: format!("Class username already exists: {}", dto.username),
            });
        }

        if let Some(image_data) = dto.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;

            dto.image_id = Some(cloud_res.public_id);
            dto.image = Some(cloud_res.secure_url);
        }

        if let Some(background_images_data) = dto.background_images.clone() {
            let mut uploaded_images = Vec::new();
            for bg in background_images_data {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url)
                    .await
                    .map_err(|e| AppError { message: e })?;
                uploaded_images.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            dto.background_images = Some(uploaded_images);
        }

        dto.code = Some(generate_code());
        dto.is_active = Some(true);

        let full_doc = bson::to_document(&dto).map_err(|e| AppError {
            message: format!("Failed to serialize class: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<Class>(extract_valid_fields(full_doc), None)
            .await
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Class, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Class>(filter, None).await?.ok_or(AppError {
            message: "Class not found".into(),
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
    ) -> Result<Paginated<Class>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "username",
            "code",
            "tags",
            "school_id",
            "creator_id",
            "class_teacher_id",
            "type",
            "is_active",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Class>(filter, &searchable, limit, skip, extra_match)
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
    pub async fn update(&self, id: &IdType, update: &UpdateClass) -> Result<Class, AppError> {
        if let Some(ref username) = update.username {
            is_valid_username(username).map_err(|e| AppError { message: e })?;
        }

        let existing_class = self.find_one(Some(id), None).await?;

        if let Some(ref username) = update.username {
            if existing_class.username != *username {
                if let Ok(class) = self
                    .find_one(None, Some(doc! { "username": username }))
                    .await
                {
                    return Err(AppError {
                        message: format!("Username already exists: {}", class.username),
                    });
                }
            }
        }

        if let Some(code) = update.code.clone().flatten() {
            if existing_class.code.as_deref() != Some(&code) {
                if let Ok(school) = self.find_one(None, Some(doc! { "code": code })).await {
                    return Err(AppError {
                        message: format!("Class code already exists: {:?}", school.code),
                    });
                }
            }
        }

        let mut update_class = update.clone();

        if let Some(new_image) = update.image.clone().flatten() {
            if Some(new_image.clone()) != existing_class.image {
                if let Some(old_image_id) = existing_class.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_class.image_id = Some(Some(cloud_res.public_id));
                update_class.image = Some(Some(cloud_res.secure_url));
            }
        }

        if let Some(Some(bg_images)) = update.background_images.clone() {
            if let Some(old_bgs) = existing_class.background_images.clone() {
                for bg in old_bgs {
                    let _ = CloudinaryService::delete_from_cloudinary(&bg.id).await;
                }
            }

            let mut uploaded_bgs: Vec<Image> = Vec::new();
            for bg in bg_images {
                let cloud_res = CloudinaryService::upload_to_cloudinary(&bg.url)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to upload background image: {}", e),
                    })?;

                uploaded_bgs.push(Image {
                    id: cloud_res.public_id,
                    url: cloud_res.secure_url,
                });
            }
            update_class.background_images = Some(Some(uploaded_bgs));
        }

        let full_doc = bson::to_document(&update_class).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Class>(id, extract_valid_fields(full_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Class, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let class = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(class)
    }

    // =========================
    // WITH RELATIONS
    // =========================
    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<ClassWithOthers>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let mut or_conditions = vec![
                doc! { "name": { "$regex": &f, "$options": "i" } },
                doc! { "username": { "$regex": &f, "$options": "i" } },
                doc! { "code": { "$regex": &f, "$options": "i" } },
                doc! { "tags": { "$in": [&f] } },
            ];

            if let Ok(oid) = ObjectId::parse_str(&f) {
                or_conditions.extend(vec![
                    doc! { "_id": oid },
                    doc! { "school_id": oid },
                    doc! { "creator_id": oid },
                    doc! { "class_teacher_id": oid },
                    doc! { "main_class_id": oid },
                    doc! { "trade_id": oid },
                ]);
            }

            match_stage.insert("$or", or_conditions);
        }

        let pipeline = class_pipeline(match_stage);
        repo.aggregate_with_paginate::<ClassWithOthers>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<ClassWithOthers, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.aggregate_one::<ClassWithOthers>(class_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Class not found".into(),
            })
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count_classes(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "school_id", "type", "is_active", "tags"];

        repo.count(filter, &searchable, extra_match).await
    }
}
