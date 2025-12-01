use mongodb::bson::{Bson, Document};

pub fn extract_valid_fields(update: Document) -> Document {
    let mut cleaned = Document::new();

    for (k, v) in update {
        match &v {
            // Skip null values
            Bson::Null => continue,

            // Skip empty strings: ""
            Bson::String(s) if s.trim().is_empty() => continue,

            // Skip empty arrays: []
            Bson::Array(a) if a.is_empty() => continue,

            // Skip empty documents: {}
            Bson::Document(d) if d.is_empty() => continue,

            // Otherwise, keep the field
            _ => cleaned.insert(k, v),
        };
    }

    cleaned
}
