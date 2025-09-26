use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::subjects::subject_category::SubjectTypeFor, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectLearningMaterial {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    pub material_type: SubjectMaterialType,
    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
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
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectMaterialType {
    Book,
    Article,
    Video,
    Note,
    ExternalLink,
    Document,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSubjectLearningMaterial {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_type: Option<SubjectMaterialType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<SubjectTypeFor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}
