use crate::{
    domain::common_details::RelatedUser, helpers::object_id_helpers, make_partial,
    schema::common_schema::ActorRef,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Like {
        #[serde(
            rename = "_id",
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub id: Option<ObjectId>,

        pub actor: ActorRef,

        #[serde(
            serialize_with = "object_id_helpers::serialize_oid",
            deserialize_with = "object_id_helpers::deserialize_oid"
        )]
        pub target_id: ObjectId,
        pub like: Option<String>,

        #[serde(default)]
        pub created_at: Option<DateTime<Utc>>,

        #[serde(default)]
        pub updated_at: Option<DateTime<Utc>>,
    } => LikePartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LikeWithRelations {
    #[serde(flatten)]
    pub like: Like,
    pub author_user: Option<RelatedUser>,
}
