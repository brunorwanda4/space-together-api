use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        class::Class,
        school::School,
        subjects::subject_category::SubjectCategory,
        teacher::Teacher,
        template_subject::{TemplateSubject, TemplateTopic},
    },
    helpers::object_id_helpers,
    make_partial,
};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct ClassSubject {
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
    pub teacher_id: Option<ObjectId>,

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
    pub main_subject_id:  Option<ObjectId>, // this is template schema id

    pub name: String,
    pub code: String,
    pub description: String,
    pub category: SubjectCategory,
    pub estimated_hours: i32,
    pub credits: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub topics: Option<Vec<TemplateTopic>>,

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
} => ClassSubjectPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassSubjectWithRelations {
    #[serde(flatten)]
    pub subject: ClassSubject,

    pub main_template_subject: Option<TemplateSubject>,
    pub school: Option<School>,
    pub class: Option<Class>,
    pub teacher: Option<Teacher>,
}
