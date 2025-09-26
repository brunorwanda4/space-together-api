use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::subjects::{
        subject_category::SubjectTypeFor, subject_competency_block::SubjectCompetencyBlock,
    },
    helpers::object_id_helpers,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningOutcome {
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
    pub subject_id: Option<ObjectId>, // Reference to MainSubject

    pub title: String,
    pub description: Option<String>,
    pub order: i32,
    pub estimated_hours: Option<i32>,
    pub key_competencies: SubjectCompetencyBlock,
    pub assessment_criteria: Vec<String>,
    pub role: SubjectTypeFor,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub prerequisites: Option<Vec<ObjectId>>, // Reference other outcomes
    pub is_mandatory: Option<bool>,

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
pub struct UpdateLearningOutcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<ObjectId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_competencies: Option<SubjectCompetencyBlock>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment_criteria: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<SubjectTypeFor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<Vec<ObjectId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mandatory: Option<bool>,
}
