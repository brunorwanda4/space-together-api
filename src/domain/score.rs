use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Score {
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
        pub student_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub class_subject_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub exam_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub assessment_category_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub education_year_id: Option<ObjectId>,

        pub score: f64,
        pub max_score: f64,
        pub percentage: f64, // Calculated: (score / max_score) * 100

        pub remarks: Option<String>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub entered_by: Option<ObjectId>, // Teacher who entered

        #[serde(default)]
        pub created_at: Option<DateTime<Utc>>,

        #[serde(default)]
        pub updated_at: Option<DateTime<Utc>>,

        #[serde(default)]
        pub is_deleted: bool,
    } => ScorePartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreWithRelations {
    #[serde(flatten)]
    pub score: Score,

    pub school: Option<crate::domain::school::School>,
    pub student: Option<crate::domain::student::Student>,
    pub class_subject: Option<crate::domain::class_subject::ClassSubject>,
    pub exam: Option<crate::domain::exam::Exam>,
    pub assessment_category: Option<crate::domain::assessment_category::AssessmentCategory>,
    pub entered_by_user: Option<crate::domain::user::User>,
}

// Audit log for score changes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreAuditLog {
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
    pub score_id: Option<ObjectId>,

    pub old_score: f64,
    pub new_score: f64,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub changed_by: Option<ObjectId>,

    pub change_reason: Option<String>,

    #[serde(default)]
    pub changed_at: Option<DateTime<Utc>>,
}
