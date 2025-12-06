use mongodb::{
    bson::{self, doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        class_subject::{ClassSubject, ClassSubjectPartial, ClassSubjectWithRelations},
        common_details::Paginated,
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::IndexDef},
    pipeline::class_subject_pipeline::class_subject_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct ClassSubjectService {
    pub collection: Collection<ClassSubject>,
}

impl ClassSubjectService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<ClassSubject>("class_subjects"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            // single-field
            IndexDef::single("code", true),
            IndexDef::single("school_id", false),
            IndexDef::single("class_id", false),
            IndexDef::single("teacher_id", false),
            IndexDef::single("main_subject_id", false),
            IndexDef::single("name", false),
            // compound
            IndexDef::compound(vec![("school_id", 1), ("class_id", 1)], false),
            IndexDef::compound(vec![("school_id", 1), ("teacher_id", 1)], false),
            IndexDef::compound(vec![("school_id", 1), ("disable", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let _ = repo.ensure_indexes(&indexes).await?;

        Ok(())
    }

    // CREATE
    pub async fn create(&self, dto: ClassSubject) -> Result<ClassSubject, AppError> {
        self.ensure_indexes().await?;
        if let Ok(sub) = self.find_one_by_code(&dto.code).await {
            return Err(AppError {
                message: format!("Class subject code already exists: {}", sub.code),
            });
        }
        let full_doc = bson::to_document(&dto.to_partial()).map_err(|e| AppError {
            message: format!("Failed to serialize create: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<ClassSubject>(full_doc, Some(&["code"])).await
    }

    // FIND BY ID
    pub async fn find_one_by_id(&self, id: &IdType) -> Result<ClassSubject, AppError> {
        let obj = IdType::to_object_id(id)?;
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! {"_id": obj};
        let sub = repo.find_one::<ClassSubject>(filter, None).await?;

        match sub {
            Some(s) => Ok(s),
            None => Err(AppError {
                message: "Class subject not found".to_string(),
            }),
        }
    }

    // FIND BY CODE
    pub async fn find_one_by_code(&self, code: &str) -> Result<ClassSubject, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! {"code": code};
        let sub = repo.find_one::<ClassSubject>(filter, None).await?;

        match sub {
            Some(s) => Ok(s),
            None => Err(AppError {
                message: "Class subject not found by code".to_string(),
            }),
        }
    }

    // GET ALL
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<ClassSubject>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "_id",
            "name",
            "code",
            "description",
            "category",
            "estimated_hours",
            "class_id",
            "school_id",
            "teacher_id",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<ClassSubject>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // UPDATE
    pub async fn update_subject(
        &self,
        id: &IdType,
        update: &ClassSubjectPartial,
    ) -> Result<ClassSubject, AppError> {
        // Validate duplicate code
        if let Some(code) = update.code.clone() {
            if let Ok(sub) = self.find_one_by_code(&code).await {
                if sub.code != code {
                    return Err(AppError {
                        message: format!("Code '{}' already exists", sub.code),
                    });
                }
            }
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let full_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        let update_doc = extract_valid_fields(full_doc);

        repo.update_one_and_fetch::<ClassSubject>(id, update_doc)
            .await
    }

    // DELETE
    pub async fn delete_subject(&self, id: &IdType) -> Result<ClassSubject, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let existing = self.find_one_by_id(id).await?;

        repo.delete_one(id).await?;

        Ok(existing)
    }

    // AGGREGATED GET ALL (with relations)
    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<ClassSubjectWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut pipeline = vec![];

        // add extra match first
        if let Some(extra) = extra_match {
            pipeline.push(doc! { "$match": extra });
        }

        // optional filter search
        if let Some(f) = filter {
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        {"_id": {"$regex": &f, "$options": "i"}},
                        {"name": {"$regex": &f, "$options": "i"}},
                        {"code": {"$regex": &f, "$options": "i"}},
                        {"description": {"$regex": &f, "$options": "i"}}
                    ]
                }
            });
        }

        // relation joins
        pipeline.extend(class_subject_pipeline(doc! {}));

        repo.aggregate_with_paginate::<ClassSubjectWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_many_and_others_by_class_id(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        class_id: &IdType,
    ) -> Result<Paginated<ClassSubjectWithRelations>, AppError> {
        let obj = IdType::to_object_id(class_id)?;
        let extra_match = doc! { "class_id": obj };

        self.get_all_with_relations(filter, limit, skip, Some(extra_match))
            .await
    }

    // GET ONE WITH RELATIONS
    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
    ) -> Result<ClassSubjectWithRelations, AppError> {
        let obj = IdType::to_object_id(id)?;
        self.find_one_with_match(doc! { "_id": obj }).await
    }

    pub async fn find_one_with_relations_by_code(
        &self,
        code: &str,
    ) -> Result<ClassSubjectWithRelations, AppError> {
        self.find_one_with_match(doc! { "code": code }).await
    }

    async fn find_one_with_match(
        &self,
        match_stage: Document,
    ) -> Result<ClassSubjectWithRelations, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let pipeline = class_subject_pipeline(match_stage);

        let result = repo
            .aggregate_one::<ClassSubjectWithRelations>(pipeline, None)
            .await?;

        match result {
            Some(v) => Ok(v),
            None => Err(AppError {
                message: "Class subject not found".to_string(),
            }),
        }
    }

    pub async fn find_many_by_teacher(
        &self,
        teacher_id: &IdType,
    ) -> Result<Vec<ClassSubject>, AppError> {
        let obj = IdType::to_object_id(teacher_id)?;

        let extra_match = doc! { "class_teacher_id": obj };

        let res = self.get_all(None, None, None, Some(extra_match)).await?;
        Ok(res.data)
    }

    pub async fn find_many_by_class_id(
        &self,
        class_id: &IdType,
    ) -> Result<Vec<ClassSubject>, AppError> {
        let obj = IdType::to_object_id(class_id)?;

        let extra_match = doc! { "class_id": obj };

        let res = self.get_all(None, None, None, Some(extra_match)).await?;
        Ok(res.data)
    }

    pub async fn create_many(
        &self,
        dtos: Vec<ClassSubject>,
    ) -> Result<Vec<ClassSubject>, AppError> {
        self.ensure_indexes().await?;
        let docs = dtos
            .into_iter()
            .map(|dto| bson::to_document(&dto.to_partial()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError {
                message: format!("Failed to serialize DTO: {}", e),
            })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create_many::<ClassSubject>(docs, Some(&["code"]))
            .await
    }
}
