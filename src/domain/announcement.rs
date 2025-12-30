use crate::{
    domain::{
        class::Class,
        common_details::{RelatedUser, UserRole},
    },
    helpers::object_id_helpers,
    make_partial,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mention {
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid",
        default
    )]
    pub id: ObjectId,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Published {
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid",
        default
    )]
    pub id: ObjectId,
    pub role: UserRole,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Announcement {
        #[serde(
            rename = "_id",
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub id: Option<ObjectId>,
        pub content: String,

    pub mention: Option<Vec<Mention>>,
    pub published: Published,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        default
    )]
    pub class_id: Option<ObjectId>,

    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    } => AnnouncementPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnouncementWithRelations {
    #[serde(flatten)]
    pub announcement: Announcement,

    // ========================
    // RELATIONS (READ-ONLY)
    // ========================
    pub published_user: Option<RelatedUser>,
    pub mentioned_users: Option<Vec<RelatedUser>>,
    pub class: Option<Class>,
}
