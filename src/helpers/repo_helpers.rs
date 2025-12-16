use crate::errors::AppError;
use mongodb::{bson::Document, Collection, IndexModel};
use serde::de::DeserializeOwned;
use serde_json::Value;
/// Helper function to safely create an index
pub async fn safe_create_index(
    collection: &Collection<Document>,
    model: IndexModel,
    name: &str,
) -> Result<(), AppError> {
    if let Err(e) = collection.create_index(model).await {
        let msg = e.to_string();
        if msg.contains("already exists") || msg.contains("IndexKeySpecsConflict") {
            println!("⚠️ Skipping existing/conflicting index: {}", name);
            Ok(())
        } else {
            Err(AppError {
                message: format!("Failed to create {} index: {}", name, msg),
            })
        }
    } else {
        // println!("✅ Created index: {}", name);
        Ok(())
    }
}

pub fn debug_deserialize_error<T: DeserializeOwned>(raw_json: &str) {
    println!("🔍 RAW JSON RECEIVED:\n{}\n", raw_json);

    let json_value: Value = match serde_json::from_str(raw_json) {
        Ok(v) => v,
        Err(e) => {
            println!("❌ JSON is not valid: {}", e);
            return;
        }
    };

    println!("🔍 JSON Parsed as Value:\n{:#?}\n", json_value);

    match serde_json::from_value::<T>(json_value.clone()) {
        Ok(_) => println!("✅ Deserialization successful"),
        Err(e) => {
            println!("❌ Deserialization Error:\n{}\n", e);

            println!("🔍 Attempting to locate problematic field...\n");

            if let Some(field) = locate_error_field(&json_value, &format!("{}", e)) {
                println!("❌ PROBLEM IS IN FIELD: {}", field);
            } else {
                println!("⚠ Could not detect exact field — check structure mismatch.");
            }
        }
    }
}

fn locate_error_field(value: &Value, err: &str) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if err.contains(k) {
                    return Some(k.clone());
                }
                // recursive search
                if let Some(inner) = locate_error_field(v, err) {
                    return Some(format!("{}.{}", k, inner));
                }
            }
        }
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                if let Some(inner) = locate_error_field(item, err) {
                    return Some(format!("[{}].{}", i, inner));
                }
            }
        }
        _ => {}
    }
    None
}
