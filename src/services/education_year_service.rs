use chrono::{DateTime, Utc};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        education_year::{EducationYear, EducationYearPartial, EducationYearWithOthers, Term},
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::IndexDef},
    pipeline::academic_year_pipeline::academic_year_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct EducationYearService {
    pub collection: Collection<EducationYear>,
}

impl EducationYearService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<EducationYear>("education_years"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("label", true),
            IndexDef::single("curriculum_id", false),
            IndexDef::single("start_date", false),
            IndexDef::single("end_date", false),
            IndexDef::compound(vec![("curriculum_id", 1), ("label", 1)], true),
            IndexDef::compound(vec![("start_date", 1), ("end_date", 1)], false),
            IndexDef::compound(vec![("curriculum_id", 1), ("start_date", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let _ = repo.ensure_indexes(&indexes).await?;

        Ok(())
    }

    // ============================================
    // CREATE
    // ============================================
    pub async fn create(&self, dto: EducationYear) -> Result<EducationYear, AppError> {
        self.ensure_indexes().await?;

        if let Ok(existing) = self
            .find_by_label_and_curriculum(&dto.label, dto.curriculum_id)
            .await
        {
            return Err(AppError {
                message: format!(
                    "Academic year {} already exists under this curriculum",
                    existing.label
                ),
            });
        }

        let new_doc = dto.to_partial();
        let full_doc = bson::to_document(&new_doc).map_err(|e| AppError {
            message: format!("Failed to serialize create: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<EducationYear>(full_doc, Some(&["label"]))
            .await
    }

    // ============================================
    // FIND BY ID
    // ============================================
    pub async fn find_one_by_id(&self, id: &IdType) -> Result<EducationYear, AppError> {
        let obj = IdType::to_object_id(id)?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let filter = doc! { "_id": obj };

        match repo.find_one::<EducationYear>(filter, None).await? {
            Some(item) => Ok(item),
            None => Err(AppError {
                message: "Academic year not found".to_string(),
            }),
        }
    }

    // ============================================
    // FIND by label + curriculum
    // ============================================
    pub async fn find_by_label_and_curriculum(
        &self,
        label: &str,
        curriculum_id: ObjectId,
    ) -> Result<EducationYear, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! {
            "label": label,
            "curriculum_id": curriculum_id
        };

        match repo.find_one::<EducationYear>(filter, None).await? {
            Some(item) => Ok(item),
            None => Err(AppError {
                message: "Academic year not found".to_string(),
            }),
        }
    }

    // ============================================
    // GET ALL (LIST)
    // ============================================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<EducationYear>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["label", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<EducationYear>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // ============================================
    // UPDATE
    // ============================================
    pub async fn update_year(
        &self,
        id: &IdType,
        update: &EducationYearPartial,
    ) -> Result<EducationYear, AppError> {
        // Uniqueness check
        if let (Some(label), Some(curriculum_id)) = (update.label.clone(), update.curriculum_id) {
            if let Ok(existing) = self
                .find_by_label_and_curriculum(&label, curriculum_id)
                .await
            {
                if existing.id != IdType::to_object_id(id).ok() {
                    return Err(AppError {
                        message: format!(
                            "Academic year {} already exists for this curriculum",
                            existing.label
                        ),
                    });
                }
            }
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let partial_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        let update_doc = extract_valid_fields(partial_doc);

        repo.update_one_and_fetch::<EducationYear>(id, update_doc)
            .await
    }

    // ============================================
    // DELETE
    // ============================================
    pub async fn delete_year(&self, id: &IdType) -> Result<EducationYear, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let found = self.find_one_by_id(id).await?;
        repo.delete_one(id).await?;

        Ok(found)
    }

    // ============================================
    // WITH RELATIONS
    // ============================================
    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<EducationYearWithOthers>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut pipeline = vec![];

        if let Some(f) = filter {
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "_id": { "$regex": &f, "$options": "i" }},
                        { "label": { "$regex": &f, "$options": "i" }}
                    ]
                }
            });
        }

        pipeline.extend(academic_year_pipeline(doc! {}));

        repo.aggregate_with_paginate::<EducationYearWithOthers>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: &IdType,
    ) -> Result<EducationYearWithOthers, AppError> {
        let obj = IdType::to_object_id(id)?;
        let match_stage = doc! { "_id": obj };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let pipeline = academic_year_pipeline(match_stage);

        let result = repo
            .aggregate_one::<EducationYearWithOthers>(pipeline, None)
            .await?;

        match result {
            Some(item) => Ok(item),
            None => Err(AppError {
                message: "Academic year not found".to_string(),
            }),
        }
    }

    pub async fn get_current_year_and_term(
        &self,
        date: Option<DateTime<Utc>>,
    ) -> Result<(EducationYear, Option<Term>), AppError> {
        let target_date = date.unwrap_or_else(Utc::now);

        let match_doc = doc! {
            "start_date": { "$lte": bson::to_bson(&target_date).unwrap() },
            "end_date":   { "$gte": bson::to_bson(&target_date).unwrap() }
        };

        let year_paginated = self
            .get_all(None, Some(1), Some(0), Some(match_doc))
            .await?;

        let year = year_paginated.data.into_iter().next().ok_or(AppError {
            message: "No active education year found for this date".to_string(),
        })?;

        let current_term = year
            .terms
            .iter()
            .find(|t| t.start_date <= target_date && t.end_date >= target_date)
            .cloned();

        Ok((year, current_term))
    }
}
