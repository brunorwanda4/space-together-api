use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReferenceIdsRequest {
    pub reference_ids: Vec<String>,
}
