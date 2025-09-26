use crate::{
    domain::subjects::subject_grading_schemes::{
        SubjectGradingScheme, SubjectGradingType, UpdateSubjectGradingScheme,
    },
    models::id_model::IdType,
    repositories::subjects::subject_grading_schemes_repo::SubjectGradingSchemesRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use std::collections::HashMap;

pub struct SubjectGradingSchemesService<'a> {
    repo: &'a SubjectGradingSchemesRepo,
}

impl<'a> SubjectGradingSchemesService<'a> {
    pub fn new(repo: &'a SubjectGradingSchemesRepo) -> Self {
        Self { repo }
    }

    /// Get all grading schemes
    pub async fn get_all_schemes(&self) -> Result<Vec<SubjectGradingScheme>, String> {
        self.repo.get_all_schemes().await.map_err(|e| e.message)
    }

    /// Create a new grading scheme
    pub async fn create_scheme(
        &self,
        mut new_scheme: SubjectGradingScheme,
    ) -> Result<SubjectGradingScheme, String> {
        // ✅ Check if scheme already exists for this subject and role
        if let (Some(main_subject_id), role) =
            (new_scheme.main_subject_id.as_ref(), new_scheme.role.clone())
        {
            let subject_id_type = IdType::from_object_id(main_subject_id.clone());
            if let Ok(Some(_)) = self
                .repo
                .find_by_subject_and_role(&subject_id_type, &role)
                .await
            {
                return Err("Grading scheme already exists for this subject and role".to_string());
            }
        }

        // ✅ Validate grading scheme data
        if let Err(validation_error) = self.validate_grading_scheme(&new_scheme) {
            return Err(validation_error);
        }

        let now = Some(Utc::now());
        new_scheme.created_at = now;
        new_scheme.updated_at = now;

        // Ensure Mongo generates id
        new_scheme.id = Some(ObjectId::new());

        let inserted_scheme = self
            .repo
            .insert_scheme(&new_scheme)
            .await
            .map_err(|e| e.message)?;
        Ok(inserted_scheme)
    }

    /// Get grading scheme by ID
    pub async fn get_scheme_by_id(&self, id: &IdType) -> Result<SubjectGradingScheme, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Grading scheme not found".to_string())
    }

    /// Get grading scheme by main_subject_id
    pub async fn get_scheme_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<SubjectGradingScheme, String> {
        self.repo
            .find_by_main_subject_id(main_subject_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Grading scheme not found for this subject".to_string())
    }

    /// Get grading scheme by main_subject_id and role
    pub async fn get_scheme_by_subject_and_role(
        &self,
        main_subject_id: &IdType,
        role: &crate::domain::subjects::subject_category::SubjectTypeFor,
    ) -> Result<SubjectGradingScheme, String> {
        self.repo
            .find_by_subject_and_role(main_subject_id, role)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Grading scheme not found for this subject and role".to_string())
    }

    /// Get grading schemes by scheme_type
    pub async fn get_schemes_by_type(
        &self,
        scheme_type: &SubjectGradingType,
    ) -> Result<Vec<SubjectGradingScheme>, String> {
        self.repo
            .find_by_scheme_type(scheme_type)
            .await
            .map_err(|e| e.message)
    }

    /// Update a grading scheme by id
    pub async fn update_scheme(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectGradingScheme,
    ) -> Result<SubjectGradingScheme, String> {
        // Fetch existing scheme
        let scheme = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Grading scheme not found".to_string())?;

        // ✅ Validate updated data if provided
        if let Some(ref grade_boundaries) = updated_data.grade_boundaries {
            if let Err(validation_error) =
                self.validate_grade_boundaries(grade_boundaries, &scheme.scheme_type)
            {
                return Err(validation_error);
            }
        }

        if let Some(ref assessment_weights) = updated_data.assessment_weights {
            if let Err(validation_error) = self.validate_assessment_weights(assessment_weights) {
                return Err(validation_error);
            }
        }

        // ✅ Validate minimum passing grade if provided
        if let Some(ref min_grade) = updated_data.minimum_passing_grade {
            if let Err(validation_error) =
                self.validate_minimum_passing_grade(min_grade, &scheme.grade_boundaries)
            {
                return Err(validation_error);
            }
        }

        let updated_scheme = self
            .repo
            .update_scheme(id, updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_scheme)
    }

    /// Delete a grading scheme by id
    pub async fn delete_scheme(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_scheme(id).await.map_err(|e| e.message)
    }

    /// Bulk get schemes by main_subject_ids
    pub async fn get_schemes_by_main_subject_ids(
        &self,
        main_subject_ids: &[ObjectId],
    ) -> Result<Vec<SubjectGradingScheme>, String> {
        self.repo
            .find_by_main_subject_ids(main_subject_ids)
            .await
            .map_err(|e| e.message)
    }

    /// Calculate grade based on scores and weights
    pub async fn calculate_grade(
        &self,
        scheme_id: &IdType,
        scores: &HashMap<String, f32>,
    ) -> Result<(String, f32), String> {
        let scheme = self.get_scheme_by_id(scheme_id).await?;

        // Calculate weighted total
        let mut total_score = 0.0;
        for (category, score) in scores {
            if let Some(weight) = scheme.assessment_weights.get(category) {
                total_score += score * (weight / 100.0);
            } else {
                return Err(format!("Unknown assessment category: {}", category));
            }
        }

        // Determine grade based on boundaries
        let grade = self.determine_grade(total_score, &scheme.grade_boundaries);
        Ok((grade, total_score))
    }

    /// Check if a grade is passing
    pub async fn is_passing_grade(&self, scheme_id: &IdType, grade: &str) -> Result<bool, String> {
        let scheme = self.get_scheme_by_id(scheme_id).await?;

        // For PassFail scheme, check if grade is "Pass"
        if matches!(scheme.scheme_type, SubjectGradingType::PassFail) {
            return Ok(grade.to_lowercase() == "pass");
        }

        // For other schemes, compare with minimum passing grade
        Ok(self.compare_grades(
            grade,
            &scheme.minimum_passing_grade,
            &scheme.grade_boundaries,
        ))
    }

    /// Get or create default scheme for a subject
    pub async fn get_or_create_default_scheme(
        &self,
        main_subject_id: &IdType,
        created_by: Option<ObjectId>,
    ) -> Result<SubjectGradingScheme, String> {
        // Try to get existing scheme
        match self.get_scheme_by_main_subject_id(main_subject_id).await {
            Ok(scheme) => Ok(scheme),
            Err(_) => {
                // Create default letter grade scheme if not exists
                let obj_id =
                    ObjectId::parse_str(main_subject_id.as_string()).map_err(|e| e.to_string())?;
                let default_scheme =
                    SubjectGradingScheme::default_letter_grade(Some(obj_id), created_by);

                self.create_scheme(default_scheme).await
            }
        }
    }

    // Validation methods
    fn validate_grading_scheme(&self, scheme: &SubjectGradingScheme) -> Result<(), String> {
        // Validate grade boundaries
        self.validate_grade_boundaries(&scheme.grade_boundaries, &scheme.scheme_type)?;

        // Validate assessment weights
        self.validate_assessment_weights(&scheme.assessment_weights)?;

        // Validate minimum passing grade exists in boundaries
        self.validate_minimum_passing_grade(
            &scheme.minimum_passing_grade,
            &scheme.grade_boundaries,
        )?;

        // Validate weights sum to 100%
        let total_weight: f32 = scheme.assessment_weights.values().sum();
        if (total_weight - 100.0).abs() > 0.01 {
            return Err("Assessment weights must sum to 100%".to_string());
        }

        Ok(())
    }

    fn validate_grade_boundaries(
        &self,
        boundaries: &HashMap<String, f32>,
        scheme_type: &SubjectGradingType,
    ) -> Result<(), String> {
        if boundaries.is_empty() {
            return Err("Grade boundaries cannot be empty".to_string());
        }

        // Validate boundaries based on scheme type
        match scheme_type {
            SubjectGradingType::LetterGrade | SubjectGradingType::Percentage => {
                for (grade, boundary) in boundaries {
                    if *boundary < 0.0 || *boundary > 100.0 {
                        return Err(format!(
                            "Grade boundary for {} must be between 0 and 100",
                            grade
                        ));
                    }
                }
            }
            SubjectGradingType::Points => {
                for (grade, boundary) in boundaries {
                    if *boundary < 0.0 {
                        return Err(format!("Grade boundary for {} cannot be negative", grade));
                    }
                }
            }
            SubjectGradingType::PassFail => {
                if boundaries.len() != 2
                    || !boundaries.contains_key("Pass")
                    || !boundaries.contains_key("Fail")
                {
                    return Err(
                        "PassFail scheme must have exactly 'Pass' and 'Fail' boundaries"
                            .to_string(),
                    );
                }
            }
        }

        Ok(())
    }

    fn validate_assessment_weights(&self, weights: &HashMap<String, f32>) -> Result<(), String> {
        if weights.is_empty() {
            return Err("Assessment weights cannot be empty".to_string());
        }

        for (category, weight) in weights {
            if *weight < 0.0 || *weight > 100.0 {
                return Err(format!("Weight for {} must be between 0 and 100", category));
            }
        }

        Ok(())
    }

    fn validate_minimum_passing_grade(
        &self,
        min_grade: &str,
        boundaries: &HashMap<String, f32>,
    ) -> Result<(), String> {
        if !boundaries.contains_key(min_grade) {
            return Err("Minimum passing grade must exist in grade boundaries".to_string());
        }
        Ok(())
    }

    fn determine_grade(&self, score: f32, boundaries: &HashMap<String, f32>) -> String {
        let mut best_grade = "F";
        let mut highest_boundary = -1.0;

        for (grade, boundary) in boundaries {
            if score >= *boundary && *boundary > highest_boundary {
                highest_boundary = *boundary;
                best_grade = grade;
            }
        }

        best_grade.to_string()
    }

    fn compare_grades(
        &self,
        grade1: &str,
        grade2: &str,
        boundaries: &HashMap<String, f32>,
    ) -> bool {
        let boundary1 = boundaries.get(grade1).unwrap_or(&0.0);
        let boundary2 = boundaries.get(grade2).unwrap_or(&0.0);
        boundary1 >= boundary2
    }
}
