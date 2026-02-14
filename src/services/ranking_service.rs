use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::student_term_result::StudentTermResult,
    errors::AppError,
    repositories::base_repo::BaseRepository,
};

pub struct RankingService {
    pub result_collection: Collection<StudentTermResult>,
}

impl RankingService {
    pub fn new(db: &Database) -> Self {
        Self {
            result_collection: db.collection::<StudentTermResult>("student_term_results"),
        }
    }

    pub async fn calculate_rankings(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let filter = doc! {
            "class_id": class_id,
            "exam_id": exam_id
        };

        let mut cursor = self
            .result_collection
            .find(filter)
            .sort(doc! { "gpa": -1, "average_percentage": -1 })
            .await?;

        let mut results: Vec<StudentTermResult> = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        let total_students = results.len() as i32;

        // Calculate ranks with tie-breaking
        let mut current_rank = 1;
        let mut previous_gpa: Option<f64> = None;

        let repo = BaseRepository::new(
            self.result_collection
                .clone()
                .clone_with_type::<Document>(),
        );

        for (index, result) in results.iter_mut().enumerate() {
            if let Some(prev_gpa) = previous_gpa {
                if (result.gpa - prev_gpa).abs() >= 0.001 {
                    current_rank = (index as i32) + 1;
                }
            }

            result.rank_in_class = Some(current_rank);
            result.total_students = Some(total_students);
            previous_gpa = Some(result.gpa);

            // Update in database
            if let Some(result_id) = result.id {
                let update_doc = doc! {
                    "rank_in_class": current_rank,
                    "total_students": total_students
                };

                let id_type = crate::models::id_model::IdType::ObjectId(result_id);
                repo.update_one_and_fetch::<StudentTermResult>(&id_type, update_doc)
                    .await
                    .ok();
            }
        }

        Ok(results)
    }

    pub async fn get_class_rankings(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let filter = doc! {
            "class_id": class_id,
            "exam_id": exam_id
        };

        let mut cursor = self
            .result_collection
            .find(filter)
            .sort(doc! { "rank_in_class": 1 })
            .await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    pub async fn get_top_students(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        limit: i64,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let filter = doc! {
            "class_id": class_id,
            "exam_id": exam_id
        };

        let mut cursor = self
            .result_collection
            .find(filter)
            .sort(doc! { "gpa": -1, "average_percentage": -1 })
            .limit(limit)
            .await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    pub async fn get_at_risk_students(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        gpa_threshold: f64,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        let filter = doc! {
            "class_id": class_id,
            "exam_id": exam_id,
            "gpa": { "$lt": gpa_threshold }
        };

        let mut cursor = self
            .result_collection
            .find(filter)
            .sort(doc! { "gpa": 1 })
            .await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }
}
