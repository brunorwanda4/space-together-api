use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct IndexDef {
    pub fields: Vec<(String, i32)>, // field name + sort order (1=asc, -1=desc)
    pub unique: bool,
}

impl IndexDef {
    /// Create a single-field index
    pub fn single(field: &str, unique: bool) -> Self {
        Self {
            fields: vec![(field.to_string(), 1)],
            unique,
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
