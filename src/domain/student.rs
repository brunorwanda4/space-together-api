use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::common_details::{Age, Gender},
    helpers::object_id_helpers,
    make_partial,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StudentStatus {
    #[default]
    Active,
    Suspended,
    Graduated,
    Left,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Student {
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
        pub user_id: Option<ObjectId>,


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
        pub subclass_id: Option<ObjectId>,
        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub creator_id: Option<ObjectId>,

        pub name: String,
        pub email: String,
        pub phone: Option<String>,
        pub gender: Option<Gender>,
        pub image: Option<String>,
        pub image_id: Option<String>,
        pub date_of_birth: Option<Age>, // You can change to DateTime<Utc> if you prefer

        pub registration_number: Option<String>,
        pub admission_year: Option<i32>,

        #[serde(default)]
        pub status: StudentStatus,

        #[serde(default)]
        pub is_active: bool,

        #[serde(default)]
        pub tags: Vec<String>,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,

        pub deleted_at: Option<DateTime<Utc>>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub deleted_by: Option<ObjectId>,
    } => StudentPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentWithRelations {
    #[serde(flatten)]
    pub student: Student,

    pub user: Option<crate::domain::user::User>,
    pub creator: Option<crate::domain::user::User>,
    pub school: Option<crate::domain::school::School>,
    pub class: Option<crate::domain::class::Class>,
    pub subclass: Option<crate::domain::class::Class>,
}
