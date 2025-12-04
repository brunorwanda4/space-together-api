use mongodb::{
    bson::{self, doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        template_subject::{TemplateSubject, TemplateSubjectPartial, TemplateSubjectWithOthers},
    },
    errors::AppError,
    models::id_model::IdType,
    pipeline::template_subject_pipeline::template_subject_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct TemplateSubjectService {
    pub collection: Collection<TemplateSubject>,
}

impl TemplateSubjectService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<TemplateSubject>("template_subjects"),
        }
    }

    pub async fn create(&self, dto: TemplateSubject) -> Result<TemplateSubject, AppError> {
        if let Ok(sub) = self.find_one_by_code(&dto.code).await {
            return Err(AppError {
                message: format!("Subject code is ready exit try other not {}", sub.code),
            });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create(&dto, Some(&["code"])).await
    }

    pub async fn find_one_by_id(&self, id: &IdType) -> Result<TemplateSubject, AppError> {
        let obj = IdType::to_object_id(id)?;
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! { "_id": obj };
        let sub = repo.find_one::<TemplateSubject>(filter, None).await?;

        match sub {
            Some(s) => Ok(s),
            None => Err(AppError {
                message: "Template subject not found".to_string(),
            }),
        }
    }

    pub async fn find_one_by_code(&self, code: &str) -> Result<TemplateSubject, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! { "code": code };
        let sub = repo.find_one::<TemplateSubject>(filter, None).await?;

        match sub {
            Some(s) => Ok(s),
            None => Err(AppError {
                message: "Template subject not found by code".to_string(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<TemplateSubject>, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "code",
            "description",
            "category",
            "estimated_hours",
            "credits",
        ];

        let (data, total, total_pages, current_page) = base_repo
            .get_all::<TemplateSubject>(filter, &searchable, limit, skip, extra_match)
            .await?;
        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update_subject(
        &self,
        id: &IdType,
        update: &TemplateSubjectPartial,
    ) -> Result<TemplateSubject, AppError> {
        if let Some(code) = update.code.clone() {
            if let Ok(sub) = self.find_one_by_code(&code).await {
                if (sub.code != code) {
                    return Err(AppError {
                        message: format!("Subject code is ready exit try other not {}", sub.code),
                    });
                }
            }
        }

        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let full_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        // Remove null fields
        let update_doc = extract_valid_fields(full_doc);

        // Update and return the updated document
        base_repo
            .update_one_and_fetch::<TemplateSubject>(id, update_doc)
            .await
    }

    pub async fn delete_subject(&self, id: &IdType) -> Result<TemplateSubject, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let sub = self.find_one_by_id(id).await?;
        base_repo.delete_one(id).await?;

        Ok(sub)
    }

    // get subject with others

    pub async fn get_all_with_other(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<TemplateSubjectWithOthers>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Start pipeline with match from filter
        let mut pipeline = vec![];

        if let Some(f) = filter {
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": { "$regex": &f, "$options": "i" }},
                        { "code": { "$regex": &f, "$options": "i" }},
                        { "description": { "$regex": &f, "$options": "i" }},
                        { "category": { "$regex": &f, "$options": "i" }}
                    ]
                }
            });
        }

        pipeline.extend(template_subject_pipeline(doc! {}));

        pipeline.push(doc! { "$sort": { "created_at": -1 } });

        repo.aggregate_with_paginate::<TemplateSubjectWithOthers>(pipeline, limit, skip)
            .await
    }

    async fn find_one_with_match(
        &self,
        match_condition: Document,
    ) -> Result<TemplateSubjectWithOthers, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let pipeline = template_subject_pipeline(match_condition);

        let result = repo
            .aggregate_one::<TemplateSubjectWithOthers>(pipeline, None)
            .await?;

        match result {
            Some(item) => Ok(item),
            None => Err(AppError {
                message: "Template subject not found".to_string(),
            }),
        }
    }

    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
    ) -> Result<TemplateSubjectWithOthers, AppError> {
        let obj = IdType::to_object_id(id)?;
        self.find_one_with_match(doc! { "_id": obj }).await
    }

    pub async fn find_one_with_relations_by_code(
        &self,
        code: &str,
    ) -> Result<TemplateSubjectWithOthers, AppError> {
        self.find_one_with_match(doc! { "code": code }).await
    }
}
