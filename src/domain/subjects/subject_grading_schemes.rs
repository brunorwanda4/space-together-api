use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::subjects::subject_category::SubjectTypeFor, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectGradingScheme {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub main_subject_id: Option<ObjectId>,
    pub scheme_type: SubjectGradingType,
    pub grade_boundaries: HashMap<String, f32>,
    pub assessment_weights: HashMap<String, f32>,
    pub minimum_passing_grade: String,
    pub role: SubjectTypeFor,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub created_by: Option<ObjectId>,

    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectGradingType {
    LetterGrade,
    Percentage,
    Points,
    PassFail,
}

// Update struct for partial updates
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSubjectGradingScheme {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme_type: Option<SubjectGradingType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade_boundaries: Option<HashMap<String, f32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment_weights: Option<HashMap<String, f32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_passing_grade: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<SubjectTypeFor>,
}

// Default grading schemes for common scenarios
impl SubjectGradingScheme {
    pub fn default_letter_grade(
        main_subject_id: Option<ObjectId>,
        created_by: Option<ObjectId>,
    ) -> Self {
        let mut grade_boundaries = HashMap::new();
        grade_boundaries.insert("A".to_string(), 90.0);
        grade_boundaries.insert("B".to_string(), 80.0);
        grade_boundaries.insert("C".to_string(), 70.0);
        grade_boundaries.insert("D".to_string(), 60.0);
        grade_boundaries.insert("F".to_string(), 0.0);

        let mut assessment_weights = HashMap::new();
        assessment_weights.insert("exams".to_string(), 40.0);
        assessment_weights.insert("assignments".to_string(), 30.0);
        assessment_weights.insert("participation".to_string(), 20.0);
        assessment_weights.insert("projects".to_string(), 10.0);

        Self {
            id: None,
            main_subject_id,
            scheme_type: SubjectGradingType::LetterGrade,
            grade_boundaries,
            assessment_weights,
            minimum_passing_grade: "D".to_string(),
            role: SubjectTypeFor::MainSubject,
            created_by,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn default_percentage(
        main_subject_id: Option<ObjectId>,
        created_by: Option<ObjectId>,
    ) -> Self {
        let mut grade_boundaries = HashMap::new();
        grade_boundaries.insert("Excellent".to_string(), 90.0);
        grade_boundaries.insert("Good".to_string(), 80.0);
        grade_boundaries.insert("Average".to_string(), 70.0);
        grade_boundaries.insert("Pass".to_string(), 60.0);
        grade_boundaries.insert("Fail".to_string(), 0.0);

        let mut assessment_weights = HashMap::new();
        assessment_weights.insert("exams".to_string(), 50.0);
        assessment_weights.insert("assignments".to_string(), 30.0);
        assessment_weights.insert("participation".to_string(), 20.0);

        Self {
            id: None,
            main_subject_id,
            scheme_type: SubjectGradingType::Percentage,
            grade_boundaries,
            assessment_weights,
            minimum_passing_grade: "Pass".to_string(),
            role: SubjectTypeFor::MainSubject,
            created_by,
            created_at: None,
            updated_at: None,
        }
    }
}
