use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MaterialType {
    #[serde(rename = "LESSON_NOTE")]
    LessonNote,
    #[serde(rename = "RESOURCE")]
    Resource,
    #[serde(rename = "VIDEO")]
    Video,
    #[serde(rename = "FILE")]
    File,
}

impl Default for MaterialType {
    fn default() -> Self {
        MaterialType::File
    }
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct LearningMaterial {
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
        pub class_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub subject_id: Option<ObjectId>,

        pub title: String,
        pub description: Option<String>,

        #[serde(default)]
        pub material_type: MaterialType,

        pub file_url: Option<String>,
        pub file_public_id: Option<String>,
        pub video_url: Option<String>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub uploaded_by: Option<ObjectId>,

        #[serde(default)]
        pub is_published: bool,

        pub deleted_at: Option<DateTime<Utc>>,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,
    } => LearningMaterialPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningMaterialWithRelations {
    #[serde(flatten)]
    pub learning_material: LearningMaterial,

    pub uploader: Option<crate::domain::user::User>,
    pub school: Option<crate::domain::school::School>,
    pub class: Option<crate::domain::class::Class>,
    pub subject: Option<crate::domain::class_subject::ClassSubject>,
}
