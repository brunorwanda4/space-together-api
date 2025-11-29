use crate::{
    domain::{
        class::Class,
        subjects::{main_subject::MainSubject, subject_category::SubjectCategory},
        teacher::Teacher,
    },
    helpers::object_id_helpers,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subject {
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
    pub username: String,

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
    pub creator_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub class_teacher_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub main_subject_id: Option<ObjectId>,

    pub subject_type: SubjectCategory,

    pub description: Option<String>,
    pub code: Option<String>,
    pub is_active: bool,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct UpdateSubject {
    pub name: Option<String>,
    pub username: Option<String>,
    pub class_id: Option<Option<ObjectId>>,
    pub class_teacher_id: Option<Option<ObjectId>>,
    pub main_subject_id: Option<Option<ObjectId>>,
    pub subject_type: Option<SubjectCategory>,
    pub description: Option<Option<String>>,
    pub code: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectWithRelations {
    #[serde(flatten)]
    pub subject: Subject,
    pub class: Option<Class>,
    pub class_teacher: Option<Teacher>,
    pub main_subject: Option<MainSubject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSubjectsRequest {
    pub subjects: Vec<Subject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSubjectsForClassRequest {
    pub subjects: Vec<Subject>,
    pub class_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSubjectsForTeacherRequest {
    pub subjects: Vec<Subject>,
    pub teacher_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSubjectsForMainSubjectRequest {
    pub subjects: Vec<Subject>,
    pub main_subject_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateRequest {
    pub updates: Vec<BulkUpdateItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateItem {
    pub id: String,
    pub update: UpdateSubject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkIdsRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkCheckIdentifiersRequest {
    pub usernames: Vec<String>,
    pub codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkCheckIdentifiersResponse {
    pub existing_usernames: Vec<String>,
    pub existing_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateActiveStatusRequest {
    pub ids: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkTagsRequest {
    pub ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateByClassRequest {
    pub class_id: String,
    pub update: UpdateSubject,
}
