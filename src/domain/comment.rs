use crate::{
    domain::common_details::RelatedUser, helpers::object_id_helpers, make_partial,
    schema::common_schema::ActorRef,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Comment {
        #[serde(
            rename = "_id",
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub id: Option<ObjectId>,

        pub content: String,

        pub author: ActorRef,
        #[serde(
            serialize_with = "object_id_helpers::serialize_oid",
            deserialize_with = "object_id_helpers::deserialize_oid"
        )]
        pub target_post_id: ObjectId,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            default
        )]
        pub parent_comment_id: Option<ObjectId>,

        /* Metadata */
        #[serde(default)]
        pub created_at: Option<DateTime<Utc>>,

        #[serde(default)]
        pub updated_at: Option<DateTime<Utc>>,
    } => CommentPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentWithRelations {
    #[serde(flatten)]
    pub comment: Comment,
    pub author_user: Option<RelatedUser>,
}
