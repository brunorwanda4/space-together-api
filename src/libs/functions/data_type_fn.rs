use mongodb::bson::{self, oid::ObjectId, Bson, Document};
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

pub fn convert_fields_to_string(doc: Document) -> Document {
    let mut new_doc = Document::new();
    for (key, value) in doc.into_iter() {
        let new_value = match value {
            Bson::ObjectId(object_id) => Bson::String(object_id.to_string()),
            Bson::DateTime(datetime) => {
                Bson::String(datetime.try_to_rfc3339_string().unwrap_or("".to_string()))
            }
            Bson::Document(sub_doc) => Bson::Document(convert_fields_to_string(sub_doc)),
            Bson::Array(arr) => {
                let converted_arr: Vec<Bson> = arr
                    .into_iter()
                    .map(|item| match item {
                        Bson::ObjectId(object_id) => Bson::String(object_id.to_string()),
                        Bson::DateTime(datetime) => {
                            Bson::String(datetime.try_to_rfc3339_string().unwrap_or("".to_string()))
                        }
                        Bson::Document(sub_doc) => {
                            Bson::Document(convert_fields_to_string(sub_doc))
                        }
                        _ => item,
                    })
                    .collect();
                Bson::Array(converted_arr)
            }
            _ => value,
        };
        new_doc.insert(key, new_value);
    }
    new_doc
}
