use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        grading_scale::{GradingScale, GradingScalePartial},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::IndexDef,
    },
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct GradingScaleService {
    pub collection: Collection<GradingScale>,
}

impl GradingScaleService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<GradingScale>("grading_scales"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(
                vec![("school_id", 1), ("education_year_id", 1), ("is_active", 1)],
                false,
            ),
            IndexDef::single("is_deleted", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(&self, scale: GradingScale) -> Result<GradingScale, AppError> {
        self.ensure_indexes().await?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let mut doc = extract_valid_fields(mongodb::bson::to_document(&scale).map_err(|e| AppError {
            message: format!("Failed to serialize grading scale: {}", e),
        })?);
        doc.insert("is_deleted", false);

        repo.create::<GradingScale>(doc, None).await
    }

    pub async fn find_one(&self, id: &IdType) -> Result<GradingScale, AppError> {
        let filter = doc! {
            "_id": IdType::to_object_id(id)?,
            "is_deleted": false
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<GradingScale>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Grading scale not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<GradingScale>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "_id",
            "school_id",
            "education_year_id",
            "grading_type",
        ];

        let mut match_filter = extra_match.unwrap_or_default();
        match_filter.insert("is_deleted", false);

        let (data, total, total_pages, current_page) = repo
            .get_all::<GradingScale>(filter, &searchable, limit, skip, Some(match_filter))
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
        update: &GradingScalePartial,
    ) -> Result<GradingScale, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = extract_valid_fields(mongodb::bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?);

        repo.update_one_and_fetch::<GradingScale>(id, update_doc)
            .await
    }

    pub async fn activate(&self, id: &IdType) -> Result<GradingScale, AppError> {
        // First get the scale to find its school_id and education_year_id
        let scale = self.find_one(id).await?;

        let school_id = scale.school_id.ok_or(AppError {
            message: "Grading scale has no school_id".into(),
        })?;

        let education_year_id = scale.education_year_id.ok_or(AppError {
            message: "Grading scale has no education_year_id".into(),
        })?;

        // Deactivate all other scales for this school and education year
        let deactivate_filter = doc! {
            "school_id": school_id,
            "education_year_id": education_year_id,
            "is_deleted": false,
            "_id": { "$ne": IdType::to_object_id(id)? }
        };
        let deactivate_update = doc! { "$set": { "is_active": false } };
        self.collection
            .update_many(deactivate_filter, deactivate_update)
            .await?;

        // Activate the selected scale
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = doc! { "is_active": true };

        repo.update_one_and_fetch::<GradingScale>(id, update_doc)
            .await
    }

    pub async fn get_active_scale(
        &self,
        school_id: &mongodb::bson::oid::ObjectId,
        education_year_id: &mongodb::bson::oid::ObjectId,
    ) -> Result<Option<GradingScale>, AppError> {
        let filter = doc! {
            "school_id": school_id,
            "education_year_id": education_year_id,
            "is_active": true,
            "is_deleted": false
        };

        let mut cursor = self.collection.find(filter).await?;
        
        if cursor.advance().await? {
            Ok(Some(cursor.deserialize_current()?))
        } else {
            Ok(None)
        }
    }

    pub fn calculate_grade(&self, scale: &GradingScale, percentage: f64) -> String {
        for boundary in &scale.grade_boundaries {
            if percentage >= boundary.min_score && percentage <= boundary.max_score {
                return boundary.grade.clone();
            }
        }
        "N/A".to_string()
    }
}
