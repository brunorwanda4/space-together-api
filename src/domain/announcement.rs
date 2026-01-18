use crate::{
    domain::{
        class::Class,
        common_details::{RelatedUser, },
    },
    helpers::object_id_helpers,
    make_partial,
    schema::common_schema::ActorRef,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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
    pub mention: Option<Vec<ActorRef>>,
    pub published: ActorRef,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
  pub classes_ids: Option<Vec<ObjectId>>,

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

    pub published_user: Option<RelatedUser>,
    pub mentioned_users: Option<Vec<RelatedUser>>,
    pub classes: Option<Vec<Class>>, // also i update this on from class into classes
}
