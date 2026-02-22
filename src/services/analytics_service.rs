use mongodb::{Collection, Database};

use crate::{
    domain::analytics::{
        AttendanceRate, EnrollmentTrend, FeeCollectionSummary, PassFailDistribution,
        TeacherWorkload,
    },
    errors::AppError,
    models::id_model::IdType,
    pipeline::analytics_pipeline::{
        attendance_rate_pipeline, enrollment_trends_pipeline, fee_collection_summary_pipeline,
        pass_fail_distribution_pipeline, teacher_workload_pipeline,
    },
    repositories::base_repo::BaseRepository,
};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;

pub struct AnalyticsService {
    pub students_collection: Collection<mongodb::bson::Document>,
    pub attendance_collection: Collection<mongodb::bson::Document>,
    pub scores_collection: Collection<mongodb::bson::Document>,
    pub finance_collection: Collection<mongodb::bson::Document>,
    pub teachers_collection: Collection<mongodb::bson::Document>,
}

impl AnalyticsService {
    pub fn new(db: &Database) -> Self {
        Self {
            students_collection: db.collection::<mongodb::bson::Document>("students"),
            attendance_collection: db.collection::<mongodb::bson::Document>("attendance"),
            scores_collection: db.collection::<mongodb::bson::Document>("scores"),
            finance_collection: db.collection::<mongodb::bson::Document>("finance"),
            teachers_collection: db.collection::<mongodb::bson::Document>("teachers"),
        }
    }

    // ========== ENROLLMENT TRENDS ==========
    pub async fn get_enrollment_trends(
        &self,
        school_id: &IdType,
        year: Option<i32>,
    ) -> Result<Vec<EnrollmentTrend>, AppError> {
        let school_oid = IdType::to_object_id(school_id)?;
        let pipeline = enrollment_trends_pipeline(school_oid, year);

        let mut cursor = self
            .students_collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to aggregate enrollment trends: {}", e),
            })?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to read cursor: {}", e),
        })? {
            let item: EnrollmentTrend =
                mongodb::bson::from_document(doc).map_err(|e| AppError {
                    message: format!("Failed to deserialize: {}", e),
                })?;
            results.push(item);
        }

        Ok(results)
    }

    // ========== ATTENDANCE RATE ==========
    pub async fn get_attendance_rate(
        &self,
        school_id: &IdType,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<AttendanceRate, AppError> {
        let school_oid = IdType::to_object_id(school_id)?;
        let pipeline = attendance_rate_pipeline(school_oid, from, to);

        let repo = BaseRepository::new(self.attendance_collection.clone());
        let result = repo
            .aggregate_one::<AttendanceRate>(pipeline, None)
            .await?;

        // Return default if no attendance records found
        Ok(result.unwrap_or(AttendanceRate {
            attendance_rate: 0.0,
            total_records: 0,
            present_count: 0,
        }))
    }

    // ========== PASS/FAIL DISTRIBUTION ==========
    pub async fn get_pass_fail_distribution(
        &self,
        school_id: &IdType,
        passing_mark: Option<f64>,
    ) -> Result<PassFailDistribution, AppError> {
        let school_oid = IdType::to_object_id(school_id)?;
        let passing_threshold = passing_mark.unwrap_or(50.0); // Default 50%

        let pipeline = pass_fail_distribution_pipeline(school_oid, passing_threshold);

        let repo = BaseRepository::new(self.scores_collection.clone());
        let result = repo
            .aggregate_one::<PassFailDistribution>(pipeline, None)
            .await?;

        // Return default if no scores found
        Ok(result.unwrap_or(PassFailDistribution {
            pass: 0,
            fail: 0,
            total: 0,
            pass_rate: 0.0,
        }))
    }

    // ========== FEE COLLECTION SUMMARY ==========
    pub async fn get_fee_summary(
        &self,
        school_id: &IdType,
    ) -> Result<FeeCollectionSummary, AppError> {
        let school_oid = IdType::to_object_id(school_id)?;
        let pipeline = fee_collection_summary_pipeline(school_oid);

        let repo = BaseRepository::new(self.finance_collection.clone());
        let result = repo
            .aggregate_one::<FeeCollectionSummary>(pipeline, None)
            .await?;

        // Return default if no finance records found
        Ok(result.unwrap_or(FeeCollectionSummary {
            total_expected: 0.0,
            total_collected: 0.0,
            total_outstanding: 0.0,
            collection_rate: 0.0,
        }))
    }

    // ========== TEACHER WORKLOAD ==========
    pub async fn get_teacher_workload(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWorkload>, AppError> {
        let school_oid = IdType::to_object_id(school_id)?;
        let pipeline = teacher_workload_pipeline(school_oid);

        let mut cursor = self
            .teachers_collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to aggregate teacher workload: {}", e),
            })?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to read cursor: {}", e),
        })? {
            let item: TeacherWorkload =
                mongodb::bson::from_document(doc).map_err(|e| AppError {
                    message: format!("Failed to deserialize: {}", e),
                })?;
            results.push(item);
        }

        Ok(results)
    }
}
