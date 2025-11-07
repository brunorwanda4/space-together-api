use mongodb::{bson::Document, Collection, IndexModel};

use crate::errors::AppError;

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
