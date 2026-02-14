use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExamType {
    Continuous,
    Midterm,
    Final,
    Quiz,
    Assignment,
    Practical,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExamStatus {
    Draft,
    Published,
    InProgress,
    Completed,
    Archived,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Exam {
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
        pub education_year_id: Option<ObjectId>,

        pub term_id: Option<String>, // Reference to Term in EducationYear

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub class_id: Option<ObjectId>, // Optional: exam for specific class

        pub name: String,
        pub description: Option<String>,
        pub exam_type: ExamType,
        pub status: ExamStatus,

        pub start_date: DateTime<Utc>,
        pub end_date: DateTime<Utc>,

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

        #[serde(default)]
        pub is_deleted: bool,
    } => ExamPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExamWithRelations {
    #[serde(flatten)]
    pub exam: Exam,

    pub school: Option<crate::domain::school::School>,
    pub education_year: Option<crate::domain::education_year::EducationYear>,
    pub class: Option<crate::domain::class::Class>,
    pub creator: Option<crate::domain::user::User>,
}
