use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectTopic {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>, // Better than string

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub learning_outcome_id: Option<ObjectId>, // Reference to parent LO

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub parent_topic_id: Option<ObjectId>, // For sub-topics (2.3.1 under 2.3)
    pub title: String,
    pub description: Option<String>,
    pub order: f32,

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
pub struct UpdateSubjectTopic {
    pub learning_outcome_id: Option<ObjectId>,

    pub parent_topic_id: Option<ObjectId>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub order: Option<f32>,
    pub created_by: Option<ObjectId>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}
