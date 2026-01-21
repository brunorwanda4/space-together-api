use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::UserRole, helpers::object_id_helpers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActorRef<R = UserRole> {
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub id: ObjectId,
    pub role: R,
}
