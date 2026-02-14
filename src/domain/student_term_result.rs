use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryScore {
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub assessment_category_id: Option<ObjectId>,
    pub category_name: String,
    pub score: f64,
    pub max_score: f64,
    pub weight_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectResult {
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub class_subject_id: Option<ObjectId>,
    pub subject_name: String,
    pub category_scores: Vec<CategoryScore>,
    pub weighted_score: f64,
    pub percentage: f64,
    pub grade: String,
    pub credits: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentTermResult {
    #[serde(
        rename = "_id",
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
    pub school_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub student_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub class_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub education_year_id: Option<ObjectId>,

    pub term_id: Option<String>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub exam_id: Option<ObjectId>,

    pub subject_results: Vec<SubjectResult>,

    pub total_score: f64,
    pub total_max_score: f64,
    pub average_percentage: f64,
    pub gpa: f64,
    pub total_credits: Option<i32>,
    pub grade: String, // Based on grading scale

    pub rank_in_class: Option<i32>,
    pub total_students: Option<i32>,

    #[serde(default)]
    pub calculated_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub is_finalized: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentTermResultWithRelations {
    #[serde(flatten)]
    pub result: StudentTermResult,

    pub student: Option<crate::domain::student::Student>,
    pub class: Option<crate::domain::class::Class>,
    pub education_year: Option<crate::domain::education_year::EducationYear>,
    pub exam: Option<crate::domain::exam::Exam>,
}
