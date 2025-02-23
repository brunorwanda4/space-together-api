use mongodb::bson::{self, oid::ObjectId, Document};
use std::str::FromStr;

/// Converts string `_id` fields into ObjectId if they are valid ObjectId strings.
pub fn convert_id_fields(mut doc: Document) -> Document {
    for (key, value) in doc.clone().into_iter() {
        if key.ends_with("_id") {
            if let bson::Bson::String(id_str) = value {
                if let Ok(object_id) = ObjectId::from_str(&id_str) {
                    doc.insert(key, bson::Bson::ObjectId(object_id));
                }
            }
        }
    }
    doc
}

pub fn convert_fields_to_string(mut doc: Document) -> Document {
    for (key, value) in doc.clone().into_iter() {
        if key.ends_with("_id") {
            if let bson::Bson::ObjectId(object_id) = value {
                doc.insert(key, bson::Bson::String(object_id.to_string()));
            }
        } else if key.ends_with("_at") {
            if let bson::Bson::DateTime(datetime) = value {
                doc.insert(
                    key,
                    bson::Bson::String(datetime.try_to_rfc3339_string().unwrap_or("".to_string())),
                );
            }
        }
    }
    doc
}
