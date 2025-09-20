use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: String,
    pub document_count: u64,
    pub size_bytes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_documents: u64,
    pub total_size_bytes: String,
    pub total_collection: usize,
    pub collections: Vec<CollectionStats>,
}
