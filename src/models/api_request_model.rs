use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReferenceIdsRequest {
    pub reference_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct RequestQuery {
    pub filter: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<i64>,
    pub is_active: Option<bool>,
}
