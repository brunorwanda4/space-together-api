use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum GradingType {
    Letter,
    Percentage,
    Competency,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradeBoundary {
    pub grade: String, // "A", "B", "Excellent", "90-100"
    pub min_score: f64,
    pub max_score: f64,
    pub gpa_value: Option<f64>, // For GPA calculation
    pub description: Option<String>,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct GradingScale {
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
        pub education_year_id: Option<ObjectId>,

        pub name: String,
        pub grading_type: GradingType,
        pub grade_boundaries: Vec<GradeBoundary>,

        #[serde(default)]
        pub is_active: bool,

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

        #[serde(default)]
        pub is_deleted: bool,
    } => GradingScalePartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradingScaleWithRelations {
    #[serde(flatten)]
    pub grading_scale: GradingScale,

    pub school: Option<crate::domain::school::School>,
    pub education_year: Option<crate::domain::education_year::EducationYear>,
    pub creator: Option<crate::domain::user::User>,
}
