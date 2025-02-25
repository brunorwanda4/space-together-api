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

pub fn convert_fields_to_string(mut doc: Document) -> Document {
    for (key, value) in doc.clone().into_iter() {
        if let Bson::ObjectId(object_id) = value {
            doc.insert(key, Bson::String(object_id.to_hex().to_string()));
        } else if let Bson::DateTime(datetime) = value {
            doc.insert(
                key,
                Bson::String(datetime.try_to_rfc3339_string().unwrap_or("".to_string())),
            );
        } else if let Bson::Document(sub_doc) = value {
            doc.insert(key, Bson::Document(convert_fields_to_string(sub_doc)));
        } else if let Bson::Array(arr) = value {
            let converted_arr: Vec<Bson> = arr
                .into_iter()
                .map(|item| {
                    if let Bson::ObjectId(object_id) = item {
                        Bson::String(object_id.to_hex().to_string())
                    } else if let Bson::DateTime(datetime) = item {
                        Bson::String(datetime.try_to_rfc3339_string().unwrap_or("".to_string()))
                    } else if let Bson::Document(sub_doc) = item {
                        Bson::Document(convert_fields_to_string(sub_doc))
                    } else {
                        item
                    }
                })
                .collect();
            doc.insert(key, Bson::Array(converted_arr));
        }
    }
    doc
}
