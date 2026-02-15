use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AssignmentStatus {
    Draft,
    #[default]
    Published,
    Archived,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SubmissionStatus {
    #[default]
    Submitted,
    Graded,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Assignment {
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

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub teacher_id: Option<ObjectId>,

        pub title: String,
        pub description: Option<String>,
        pub instructions: Option<String>,

        pub due_date: DateTime<Utc>,
        pub max_score: f64,

        #[serde(default)]
        pub allow_late_submission: bool,

        pub attachment_url: Option<String>,
        pub attachment_id: Option<String>,

        #[serde(default)]
        pub status: AssignmentStatus,

        #[serde(default)]
        pub auto_grade_enabled: bool,

        #[serde(default)]
        pub is_deleted: bool,

        pub deleted_at: Option<DateTime<Utc>>,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,
    } => AssignmentPartial
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Submission {
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
        pub assignment_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub student_id: Option<ObjectId>,

        pub file_url: Option<String>,
        pub file_id: Option<String>,
        pub comment: Option<String>,

        #[serde(default)]
        pub is_late: bool,

        pub score: Option<f64>,
        pub feedback: Option<String>,
        pub feedback_file_url: Option<String>,
        pub feedback_file_id: Option<String>,

        pub graded_at: Option<DateTime<Utc>>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub graded_by: Option<ObjectId>,

        #[serde(default)]
        pub status: SubmissionStatus,

        pub auto_grade_score: Option<f64>,
        pub ai_feedback: Option<String>,

        #[serde(default)]
        pub is_deleted: bool,

        pub deleted_at: Option<DateTime<Utc>>,

        #[serde(default = "Utc::now")]
        pub submitted_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,
    } => SubmissionPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignmentWithRelations {
    #[serde(flatten)]
    pub assignment: Assignment,

    pub teacher: Option<crate::domain::teacher::Teacher>,
    pub subject: Option<crate::domain::class_subject::ClassSubject>,
    pub class: Option<crate::domain::class::Class>,
    pub submission_count: Option<i64>,
    pub total_students: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmissionWithRelations {
    #[serde(flatten)]
    pub submission: Submission,

    pub student: Option<crate::domain::student::Student>,
    pub assignment: Option<Assignment>,
    pub graded_by_teacher: Option<crate::domain::teacher::Teacher>,
}
