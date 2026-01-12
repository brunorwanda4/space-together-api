use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestQuery {
    pub filter: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<i64>,
    pub field: Option<String>,
    pub value: Option<String>,
}

#[derive(Deserialize)]
pub struct GetByIdsBody {
    pub ids: Vec<String>,
}
