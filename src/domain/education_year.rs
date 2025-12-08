use crate::{domain::sector::Sector, helpers::object_id_helpers, make_partial};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Term {
        pub name: String,       // "Term 1"
        pub order: i32,         // 1, 2, 3
        pub start_date: DateTime<Utc>,
        pub end_date: DateTime<Utc>,
    } => TermPartial
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct EducationYear {
        #[serde(
            rename = "_id",
            // alias = "id",
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize_oid",
            deserialize_with = "object_id_helpers::deserialize_oid"
        )]
        pub curriculum_id: ObjectId,

        pub label: String,                // "2025-2026"
        pub start_date: DateTime<Utc>,
        pub end_date: DateTime<Utc>,

        pub terms: Vec<Term>,             // Embedded terms

        #[serde(default)]
        pub created_at: Option<DateTime<Utc>>,

        #[serde(default)]
        pub updated_at: Option<DateTime<Utc>>,
    } => EducationYearPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EducationYearWithOthers {
    #[serde(flatten)]
    pub academic: EducationYear,

    pub curriculum: Option<Sector>,
}
