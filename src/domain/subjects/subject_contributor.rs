use crate::helpers::object_id_helpers;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectContributor {
    pub name: String,
    pub role: String, // "Author", "Reviewer", etc.
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        default
    )]
    pub user_id: Option<ObjectId>,
}
