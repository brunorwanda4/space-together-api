use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectContributor {
    pub name: String,
    pub role: String, // "Author", "Reviewer", etc.
}
