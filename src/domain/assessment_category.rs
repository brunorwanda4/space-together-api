use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct AssessmentCategory {
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
        pub class_subject_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub education_year_id: Option<ObjectId>,

        pub name: String,
        pub code: String,
        pub weight_percentage: f64, // 0-100
        pub description: Option<String>,

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
    } => AssessmentCategoryPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssessmentCategoryWithRelations {
    #[serde(flatten)]
    pub category: AssessmentCategory,

    pub school: Option<crate::domain::school::School>,
    pub class_subject: Option<crate::domain::class_subject::ClassSubject>,
    pub education_year: Option<crate::domain::education_year::EducationYear>,
    pub creator: Option<crate::domain::user::User>,
}
