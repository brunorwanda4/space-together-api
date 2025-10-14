use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::subjects::{
        learning_outcome::LearningOutcomeWithOthers, subject_category::SubjectCategory,
        subject_contributor::SubjectContributor, subject_grading_schemes::SubjectGradingScheme,
        subject_progress_tracking_config::SubjectProgressTrackingConfig,
    },
    helpers::object_id_helpers,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainSubject {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub level: Option<String>,
    pub estimated_hours: i32,
    pub credits: Option<i32>,
    pub category: SubjectCategory,

    #[serde(
        serialize_with = "object_id_helpers::serialize_vec",
        deserialize_with = "object_id_helpers::deserialize_vec",
        default
    )]
    pub main_class_ids: Vec<ObjectId>, // this will create help to create subject which have connection on subject

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub prerequisites: Option<Vec<ObjectId>>,

    pub contributors: Vec<SubjectContributor>,
    pub starting_year: Option<DateTime<Utc>>,
    pub ending_year: Option<DateTime<Utc>>,

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
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateMainSubject {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub level: Option<String>,
    pub estimated_hours: Option<i32>,
    pub credits: Option<i32>,
    pub category: Option<SubjectCategory>,
    pub main_class_ids: Option<Vec<ObjectId>>,
    pub prerequisites: Option<Vec<ObjectId>>,
    pub contributors: Option<Vec<SubjectContributor>>,
    pub starting_year: Option<DateTime<Utc>>,
    pub ending_year: Option<DateTime<Utc>>,
    pub created_by: Option<ObjectId>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainSubjectWithOthers {
    #[serde(flatten)]
    pub subject: MainSubject,

    #[serde(default)]
    pub learning_outcome: Option<Vec<LearningOutcomeWithOthers>>,

    #[serde(default)]
    pub progress_tracking_config: Option<SubjectProgressTrackingConfig>,

    #[serde(default)]
    pub grading_schemes: Option<SubjectGradingScheme>,
}

#[derive(Deserialize)]
pub struct SubjectQuery {
    pub filter: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<i64>,
    pub is_active: Option<bool>,
}
