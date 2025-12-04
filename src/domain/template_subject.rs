use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{main_class::MainClass, subjects::subject_category::SubjectCategory, user::User},
    helpers::object_id_helpers,
    make_partial,
};

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateTopic {
    pub order: String,
    pub title: String,
    pub description: Option<String>,
    pub estimated_hours: Option<i32>,
    pub credits: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subtopics: Option<Vec<TemplateTopic>>,
} => TemplateTopicPartial
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateSubject {
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
    pub description: String,
    pub category: SubjectCategory,
    pub estimated_hours: i32,
    pub credits: Option<i32>,
    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub prerequisites: Option<Vec<ObjectId>>, // this is form main_classes collection

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
} => TemplateSubjectPartial

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateSubjectWithOthers {
    #[serde(flatten)]
    pub subject: TemplateSubject,

    pub creator_user: Option<User>,
    pub prerequisite_classes: Option<Vec<MainClass>>,
}
