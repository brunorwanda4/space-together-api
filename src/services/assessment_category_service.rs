use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        assessment_category::{AssessmentCategory, AssessmentCategoryPartial},
        common_details::Paginated,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::IndexDef,
    },
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct AssessmentCategoryService {
    pub collection: Collection<AssessmentCategory>,
}

impl AssessmentCategoryService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<AssessmentCategory>("assessment_categories"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(vec![("school_id", 1), ("class_subject_id", 1)], false),
            IndexDef::single("is_deleted", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(
        &self,
        category: AssessmentCategory,
    ) -> Result<AssessmentCategory, AppError> {
        self.ensure_indexes().await?;

        // Validate total weight doesn't exceed 100%
        self.validate_total_weight(
            &category.class_subject_id,
            &category.education_year_id,
            category.weight_percentage,
            None,
        )
        .await?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let mut doc = extract_valid_fields(mongodb::bson::to_document(&category).map_err(|e| AppError {
            message: format!("Failed to serialize category: {}", e),
        })?);
        doc.insert("is_deleted", false);

        repo.create::<AssessmentCategory>(doc, None).await
    }

    pub async fn find_one(&self, id: &IdType) -> Result<AssessmentCategory, AppError> {
        let filter = doc! {
            "_id": IdType::to_object_id(id)?,
            "is_deleted": false
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<AssessmentCategory>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Assessment category not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<AssessmentCategory>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "code",
            "description",
            "_id",
            "school_id",
            "class_subject_id",
            "education_year_id",
        ];

        let mut match_filter = extra_match.unwrap_or_default();
        match_filter.insert("is_deleted", false);

        let (data, total, total_pages, current_page) = repo
            .get_all::<AssessmentCategory>(filter, &searchable, limit, skip, Some(match_filter))
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update(
        &self,
        id: &IdType,
        update: &AssessmentCategoryPartial,
    ) -> Result<AssessmentCategory, AppError> {
        // If weight is being updated, validate total weight
        if let Some(new_weight) = update.weight_percentage {
            let existing = self.find_one(id).await?;

            self.validate_total_weight(
                &existing.class_subject_id,
                &existing.education_year_id,
                new_weight,
                Some(id),
            )
            .await?;
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = extract_valid_fields(mongodb::bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?);

        repo.update_one_and_fetch::<AssessmentCategory>(id, update_doc)
            .await
    }

    pub async fn delete(&self, id: &IdType) -> Result<AssessmentCategory, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = doc! { "is_deleted": true };

        repo.update_one_and_fetch::<AssessmentCategory>(id, update_doc)
            .await
    }

    pub async fn validate_total_weight(
        &self,
        class_subject_id: &Option<ObjectId>,
        education_year_id: &Option<ObjectId>,
        new_weight: f64,
        exclude_id: Option<&IdType>,
    ) -> Result<(), AppError> {
        let mut filter = doc! {
            "class_subject_id": class_subject_id,
            "education_year_id": education_year_id,
            "is_deleted": false
        };

        if let Some(id) = exclude_id {
            filter.insert("_id", doc! { "$ne": IdType::to_object_id(id)? });
        }

        let mut cursor = self.collection.find(filter).await?;
        let mut total_weight = new_weight;

        while cursor.advance().await? {
            let category: AssessmentCategory = cursor.deserialize_current()?;
            total_weight += category.weight_percentage;
        }

        if total_weight > 100.0 {
            return Err(AppError {
                message: format!(
                    "Total weight percentage exceeds 100%. Current total: {:.2}%",
                    total_weight
                ),
            });
        }

        Ok(())
    }

    pub async fn get_total_weight(
        &self,
        class_subject_id: &ObjectId,
        education_year_id: &ObjectId,
    ) -> Result<f64, AppError> {
        let filter = doc! {
            "class_subject_id": class_subject_id,
            "education_year_id": education_year_id,
            "is_deleted": false
        };

        let mut cursor = self.collection.find(filter).await?;
        let mut total_weight = 0.0;

        while cursor.advance().await? {
            let category: AssessmentCategory = cursor.deserialize_current()?;
            total_weight += category.weight_percentage;
        }

        Ok(total_weight)
    }
}
