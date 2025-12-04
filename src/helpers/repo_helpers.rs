use crate::errors::AppError;
use mongodb::{bson::Document, Collection, IndexModel};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;

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

pub fn set_field<T, V>(target: &mut T, field: &str, value: V) -> Result<(), AppError>
where
    T: Serialize + DeserializeOwned + Clone,
    V: Serialize,
{
    let mut v = serde_json::to_value(target.clone()).map_err(|e| AppError {
        message: format!("Serialize error: {}", e),
    })?;

    if let Some(obj) = v.as_object_mut() {
        obj.insert(field.to_string(), json!(value));
    }

    // Convert JSON back to T
    let updated: T = serde_json::from_value(v).map_err(|e| AppError {
        message: format!("Deserialize error: {}", e),
    })?;

    *target = updated;
    Ok(())
}
