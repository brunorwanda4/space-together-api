use mongodb::bson::{Bson, Document};

/// Recursively removes:
/// - null values
/// - empty strings
/// - empty arrays
/// - empty documents
///
/// Example output:
/// /// { "settings.students.auto_enroll_subclasses": false }
pub fn extract_valid_fields(doc: Document) -> Document {
    let mut out = Document::new();
    clean_document(None, doc, &mut out);
    out
}

fn clean_document(prefix: Option<String>, doc: Document, out: &mut Document) {
    for (key, value) in doc {
        let full_key = match &prefix {
            Some(p) => format!("{}.{}", p, key),
            None => key,
        };

        match value {
            // ❌ remove nulls
            Bson::Null => {}

            // ❌ empty string
            Bson::String(ref s) if s.trim().is_empty() => {}

            // ❌ empty array
            Bson::Array(ref a) if a.is_empty() => {}

            // ❌ empty document
            Bson::Document(ref d) if d.is_empty() => {}

            // 🔁 recurse into sub-documents
            Bson::Document(d) => {
                clean_document(Some(full_key), d, out);
            }

            // ✅ keep valid values
            other => {
                out.insert(full_key, other);
            }
        }
    }
}
