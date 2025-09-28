use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::helpers::object_id_helpers;

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
    pub role: SubjectLearningMaterialRole,

    /// This stores the ID of the related entity
    /// If role = MainSubject → this is a main_subject_id
    /// If role = ClassSubject → this is a class_subject_id
    /// If role = SubjectTopic → this is a subject_topic_id
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub reference_id: Option<ObjectId>,

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
pub enum SubjectLearningMaterialRole {
    MainSubject,
    ClassSubject,
    SubjectTopic,
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
    pub role: Option<SubjectLearningMaterialRole>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<ObjectId>,
}
