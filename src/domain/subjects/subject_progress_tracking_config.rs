use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectProgressTrackingConfig {
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
    pub track_attendance: bool,
    pub track_assignments: bool,
    pub track_topic_coverage: bool,
    pub track_skill_acquisition: bool,
    pub thresholds: SubjectProgressThresholds,

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
pub struct SubjectProgressThresholds {
    pub satisfactory: f64,
    pub needs_improvement: f64,
    pub at_risk: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSubjectProgressTrackingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_attendance: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_assignments: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_topic_coverage: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_skill_acquisition: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thresholds: Option<UpdateSubjectProgressThresholds>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSubjectProgressThresholds {
    pub satisfactory: Option<f64>,
    pub needs_improvement: Option<f64>,
    pub at_risk: Option<f64>,
}
