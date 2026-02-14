use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        assessment_category::AssessmentCategory,
        class_subject::ClassSubject,
        score::Score,
        student_term_result::{CategoryScore, StudentTermResult, SubjectResult},
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::IndexDef},
    repositories::base_repo::BaseRepository,
    services::grading_scale_service::GradingScaleService,
};

pub struct GpaCalculationService {
    pub score_collection: Collection<Score>,
    pub result_collection: Collection<StudentTermResult>,
    pub subject_collection: Collection<ClassSubject>,
    pub category_collection: Collection<AssessmentCategory>,
    pub student_collection: Collection<crate::domain::student::Student>,
    pub grading_service: GradingScaleService,
}

impl GpaCalculationService {
    pub fn new(db: &Database) -> Self {
        Self {
            score_collection: db.collection::<Score>("scores"),
            result_collection: db.collection::<StudentTermResult>("student_term_results"),
            subject_collection: db.collection::<ClassSubject>("class_subjects"),
            category_collection: db.collection::<AssessmentCategory>("assessment_categories"),
            student_collection: db.collection::<crate::domain::student::Student>("students"),
            grading_service: GradingScaleService::new(db),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(
                vec![("school_id", 1), ("student_id", 1), ("education_year_id", 1)],
                false,
            ),
            IndexDef::compound(vec![("school_id", 1), ("class_id", 1), ("exam_id", 1)], false),
        ];

        let repo = BaseRepository::new(
            self.result_collection
                .clone()
                .clone_with_type::<Document>(),
        );
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn calculate_student_result(
        &self,
        student_id: &ObjectId,
        exam_id: &ObjectId,
        class_id: &ObjectId,
        education_year_id: &ObjectId,
        school_id: &ObjectId,
        term_id: Option<String>,
    ) -> Result<StudentTermResult, AppError> {
        // Fetch all scores for this student and exam
        let score_filter = doc! {
            "student_id": student_id,
            "exam_id": exam_id,
            "is_deleted": false
        };

        let mut cursor = self.score_collection.find(score_filter).await?;
        let mut scores: Vec<Score> = Vec::new();

        while cursor.advance().await? {
            scores.push(cursor.deserialize_current()?);
        }

        if scores.is_empty() {
            return Err(AppError {
                message: "No scores found for this student and exam".to_string(),
            });
        }

        // Group scores by subject
        let mut subject_scores: std::collections::HashMap<ObjectId, Vec<Score>> =
            std::collections::HashMap::new();

        for score in scores {
            if let Some(subject_id) = score.class_subject_id {
                subject_scores
                    .entry(subject_id)
                    .or_insert_with(Vec::new)
                    .push(score);
            }
        }

        // Calculate subject results
        let mut subject_results = Vec::new();
        let mut total_weighted_score = 0.0;
        let mut total_credits = 0;

        for (subject_id, scores) in subject_scores {
            let subject_result = self
                .calculate_subject_result(&subject_id, scores, education_year_id)
                .await?;

            total_weighted_score += subject_result.weighted_score;
            if let Some(credits) = subject_result.credits {
                total_credits += credits;
            }

            subject_results.push(subject_result);
        }

        // Calculate overall GPA
        let subject_count = subject_results.len() as f64;
        let average_percentage = if subject_count > 0.0 {
            total_weighted_score / subject_count
        } else {
            0.0
        };

        // Calculate GPA (simple average for now, can be credit-based)
        let gpa = average_percentage / 25.0; // Convert to 4.0 scale (100/25 = 4.0)

        // Get grading scale and determine grade
        let grading_scale = self
            .grading_service
            .get_active_scale(school_id, education_year_id)
            .await?;

        let grade = if let Some(scale) = grading_scale {
            self.grading_service.calculate_grade(&scale, average_percentage)
        } else {
            "N/A".to_string()
        };

        // Create result
        let result = StudentTermResult {
            id: None,
            school_id: Some(*school_id),
            student_id: Some(*student_id),
            class_id: Some(*class_id),
            education_year_id: Some(*education_year_id),
            term_id,
            exam_id: Some(*exam_id),
            subject_results,
            total_score: total_weighted_score,
            total_max_score: subject_count * 100.0,
            average_percentage,
            gpa,
            total_credits: if total_credits > 0 {
                Some(total_credits)
            } else {
                None
            },
            grade,
            rank_in_class: None, // Will be calculated separately
            total_students: None,
            calculated_at: Some(chrono::Utc::now()),
            is_finalized: false,
        };

        // Save or update result
        self.save_result(&result).await
    }

    async fn calculate_subject_result(
        &self,
        subject_id: &ObjectId,
        scores: Vec<Score>,
        education_year_id: &ObjectId,
    ) -> Result<SubjectResult, AppError> {
        // Fetch subject details
        let subject = self
            .subject_collection
            .find_one(doc! { "_id": subject_id })
            .await?
            .ok_or_else(|| AppError {
                message: "Subject not found".to_string(),
            })?;

        // Fetch assessment categories for this subject
        let category_filter = doc! {
            "class_subject_id": subject_id,
            "education_year_id": education_year_id,
            "is_deleted": false
        };

        let mut category_cursor = self.category_collection.find(category_filter).await?;
        let mut categories: Vec<AssessmentCategory> = Vec::new();

        while category_cursor.advance().await? {
            categories.push(category_cursor.deserialize_current()?);
        }

        // Calculate weighted score
        let mut category_scores = Vec::new();
        let mut weighted_total = 0.0;

        for score in scores {
            if let Some(cat_id) = score.assessment_category_id {
                // Find matching category
                if let Some(category) = categories.iter().find(|c| c.id == Some(cat_id)) {
                    let weighted_contribution =
                        score.percentage * (category.weight_percentage / 100.0);
                    weighted_total += weighted_contribution;

                    category_scores.push(CategoryScore {
                        assessment_category_id: Some(cat_id),
                        category_name: category.name.clone(),
                        score: score.score,
                        max_score: score.max_score,
                        weight_percentage: category.weight_percentage,
                    });
                }
            }
        }

        // Determine grade based on weighted percentage
        let grade = self.determine_grade(weighted_total);

        Ok(SubjectResult {
            class_subject_id: Some(*subject_id),
            subject_name: subject.name.clone(),
            category_scores,
            weighted_score: weighted_total,
            percentage: weighted_total,
            grade,
            credits: subject.credits,
        })
    }

    fn determine_grade(&self, percentage: f64) -> String {
        // Simple grading logic (can be replaced with grading scale)
        if percentage >= 90.0 {
            "A".to_string()
        } else if percentage >= 80.0 {
            "B".to_string()
        } else if percentage >= 70.0 {
            "C".to_string()
        } else if percentage >= 60.0 {
            "D".to_string()
        } else if percentage >= 50.0 {
            "E".to_string()
        } else {
            "F".to_string()
        }
    }

    async fn save_result(&self, result: &StudentTermResult) -> Result<StudentTermResult, AppError> {
        let filter = doc! {
            "student_id": result.student_id,
            "exam_id": result.exam_id
        };

        let repo = BaseRepository::new(
            self.result_collection
                .clone()
                .clone_with_type::<Document>(),
        );

        // Check if result already exists
        if let Some(existing) = repo.find_one::<StudentTermResult>(filter.clone(), None).await? {
            // Update existing result
            let update_doc = mongodb::bson::to_document(result).map_err(|e| AppError {
                message: format!("Failed to serialize result: {}", e),
            })?;
            let id = existing.id.ok_or(AppError {
                message: "Existing result has no ID".into(),
            })?;
            let id_type = IdType::ObjectId(id);

            repo.update_one_and_fetch::<StudentTermResult>(&id_type, update_doc)
                .await
        } else {
            // Insert new result
            let doc = mongodb::bson::to_document(result).map_err(|e| AppError {
                message: format!("Failed to serialize result: {}", e),
            })?;
            repo.create::<StudentTermResult>(doc, None).await
        }
    }

    pub async fn calculate_class_results(
        &self,
        class_id: &ObjectId,
        exam_id: &ObjectId,
        education_year_id: &ObjectId,
        school_id: &ObjectId,
        term_id: Option<String>,
    ) -> Result<Vec<StudentTermResult>, AppError> {
        // Fetch all students in the class
        let student_filter = doc! {
            "class_id": class_id,
            "is_active": true
        };

        let mut student_cursor = self.student_collection.find(student_filter).await?;
        let mut results = Vec::new();

        while student_cursor.advance().await? {
            let student: crate::domain::student::Student = student_cursor.deserialize_current()?;

            if let Some(student_id) = student.id {
                match self
                    .calculate_student_result(
                        &student_id,
                        exam_id,
                        class_id,
                        education_year_id,
                        school_id,
                        term_id.clone(),
                    )
                    .await
                {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!("Failed to calculate result for student {}: {}", student_id, e.message);
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn get_student_result(
        &self,
        student_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Option<StudentTermResult>, AppError> {
        let filter = doc! {
            "student_id": student_id,
            "exam_id": exam_id
        };

        let repo = BaseRepository::new(
            self.result_collection
                .clone()
                .clone_with_type::<Document>(),
        );

        repo.find_one::<StudentTermResult>(filter, None).await
    }
}
