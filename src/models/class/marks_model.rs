use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MarkModel {
    #[serde(rename = "_id", skip_serializing_if = "Optional::is_none")]
    pub id: Option<ObjectId>,
    pub amount: u32,
    pub reason: ObjectId,
    pub activity: ObjectId,
    pub student: ObjectId,
    pub create_at: DateTime,
    pub update_at: Option<DateTime>,
}
