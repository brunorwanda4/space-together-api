use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        exam::{Exam, ExamPartial, ExamStatus},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct ExamService {
    pub collection: Collection<Exam>,
}

impl ExamService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Exam>("exams"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(
                vec![("school_id", 1), ("education_year_id", 1), ("status", 1)],
                false,
            ),
            IndexDef::compound(
                vec![("school_id", 1), ("class_id", 1), ("start_date", -1)],
                false,
            ),
            IndexDef::single("is_deleted", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(&self, exam: Exam) -> Result<Exam, AppError> {
        self.ensure_indexes().await?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let mut doc = extract_valid_fields(mongodb::bson::to_document(&exam).map_err(|e| AppError {
            message: format!("Failed to serialize exam: {}", e),
        })?);
        doc.insert("is_deleted", false);

        repo.create::<Exam>(doc, None).await
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Exam, AppError> {
        let filter = doc! {
            "_id": IdType::to_object_id(id)?,
            "is_deleted": false
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<Exam>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Exam not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Exam>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "description",
            "_id",
            "school_id",
            "education_year_id",
            "class_id",
            "term_id",
            "status",
            "exam_type",
        ];

        let mut match_filter = extra_match.unwrap_or_default();
        match_filter.insert("is_deleted", false);

        let (data, total, total_pages, current_page) = repo
            .get_all::<Exam>(filter, &searchable, limit, skip, Some(match_filter))
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update(&self, id: &IdType, update: &ExamPartial) -> Result<Exam, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = extract_valid_fields(mongodb::bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?);

        repo.update_one_and_fetch::<Exam>(id, update_doc).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Exam, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = doc! { "is_deleted": true };

        repo.update_one_and_fetch::<Exam>(id, update_doc).await
    }

    pub async fn publish(&self, id: &IdType) -> Result<Exam, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = doc! {
            "status": mongodb::bson::to_bson(&ExamStatus::Published).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?
        };

        repo.update_one_and_fetch::<Exam>(id, update_doc).await
    }

    pub async fn count_exams(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "school_id",
            "education_year_id",
            "class_id",
            "status",
        ];

        let mut match_filter = extra_match.unwrap_or_default();
        match_filter.insert("is_deleted", false);

        repo.count(filter, &searchable, Some(match_filter)).await
    }
}
