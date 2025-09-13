use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum IdType {
    ObjectId(ObjectId),
    String(String),
}

impl IdType {
    pub fn as_string(&self) -> String {
        match self {
            IdType::ObjectId(oid) => oid.to_hex(),
            IdType::String(s) => s.clone(),
        }
    }

    /// Create an IdType from ObjectId
    pub fn from_object_id(oid: ObjectId) -> Self {
        IdType::ObjectId(oid)
    }

    /// Create an IdType from String
    pub fn from_string<S: Into<String>>(s: S) -> Self {
        IdType::String(s.into())
    }
}
