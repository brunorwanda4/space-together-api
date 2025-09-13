use mongodb::{bson::oid::ObjectId, results::InsertOneResult};
use serde_json::Value;

pub fn change_insertoneresult_into_object_id(id: InsertOneResult) -> Result<ObjectId, String> {
    id.inserted_id
        .as_object_id()
        .ok_or_else(|| "Failed to convert inserted_id to ObjectId".to_string())
}

pub fn convert_object_id_to_string(mut doc: Value) -> Value {
    match &mut doc {
        Value::Object(map) => {
            // 👉 special case: whole object is {"$oid": "..."}
            if map.len() == 1 {
                if let Some(oid) = map.get("$oid").and_then(|v| v.as_str()) {
                    return Value::String(oid.to_string());
                }
            }

            // otherwise recurse normally
            for (_k, v) in map.iter_mut() {
                *v = convert_object_id_to_string(v.take());
            }
        }
        Value::Array(arr) => {
            for i in arr.iter_mut() {
                *i = convert_object_id_to_string(i.take());
            }
        }
        _ => {}
    }
    doc
}
