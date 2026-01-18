use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::UserRole, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActorRef<R = UserRole> {
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid",
        default
    )]
    pub id: ObjectId,
    pub role: R,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum PostType {
    C,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActorDetails<R = UserRole> {
    pub id: ObjectId,
    pub role: R,
}
