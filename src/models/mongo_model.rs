use mongodb::bson::Document;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct IndexDef {
    pub fields: Vec<(String, i32)>, // field name + sort order (1=asc, -1=desc)
    pub unique: bool,
    pub partial: Option<Document>,
    pub name: Option<String>,
}

impl IndexDef {
    /// Create a single-field index
    pub fn single(field: &str, unique: bool) -> Self {
        Self {
            fields: vec![(field.to_string(), 1)],
            unique,
            partial: None,
            name: None,
        }
    }

    /// Create a compound index with multiple fields
    pub fn compound(fields: Vec<(&str, i32)>, unique: bool) -> Self {
        Self {
            fields: fields
                .into_iter()
                .map(|(f, order)| (f.to_string(), order))
                .collect(),
            unique,
            partial: None,
            name: None,
        }
    }

    /// Create a single-field partial index (MongoDB partialFilterExpression)
    pub fn single_with_partial(
        field: &str,
        unique: bool,
        partial: Document,
        name: Option<&str>,
    ) -> Self {
        Self {
            fields: vec![(field.to_string(), 1)],
            unique,
            partial: Some(partial),
            name: name.map(|n| n.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MongoFields {
    pub fields: Vec<String>,
}

// count doc
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountDoc {
    pub count: u64,
}
