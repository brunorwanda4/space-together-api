use mongodb::bson::{oid::ObjectId, Bson, DateTime, Document};
use serde::Serialize;
use std::str::FromStr;

/// Generic function to convert any struct into a MongoDB Document
pub fn to_document<T: Serialize>(data: &T) -> Document {
    let bson = mongodb::bson::to_bson(data).unwrap();
    let mut doc = if let Bson::Document(d) = bson {
        d
    } else {
        Document::new()
    };

    for (key, value) in doc.clone().into_iter() {
        if key.ends_with("_id") {
            if let Bson::String(id_str) = &value {
                if let Ok(object_id) = ObjectId::from_str(id_str) {
                    doc.insert(key, Bson::ObjectId(object_id));
                }
            }
        } else if key.ends_with("_at") {
            if let Bson::String(date_str) = &value {
                if let Ok(date_time) = DateTime::parse_rfc3339_str(date_str) {
                    doc.insert(key, Bson::DateTime(date_time));
                }
            }
        }
    }

    doc
}

// Updated versions of previous functions
// pub fn create_document<T: Serialize>(data: &T) -> Document {
//     to_document(data)
// }

// pub fn update_document<T: Serialize>(data: &T) -> Document {
//     let mut doc = to_document(data);
//     doc.insert("updated_at", Bson::DateTime(DateTime::now()));
//     doc
// }

// pub fn format_document(mut doc: Document) -> Document {
//     for (key, value) in doc.clone().into_iter() {
//         match value {
//             Bson::ObjectId(object_id) if key.ends_with("_id") => {
//                 doc.insert(key, Bson::String(object_id.to_string()));
//             }
//             Bson::DateTime(date_time) if key.ends_with("_at") => {
//                 if let Ok(date_str) = date_time.try_to_rfc3339_string() {
//                     doc.insert(key, Bson::String(date_str));
//                 }
//             }
//             _ => {}
//         }
//     }
//     doc
// }
