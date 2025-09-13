use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Address {
    pub country: String,
    pub province: Option<String>,
    pub district: Option<String>,
    pub sector: Option<String>,
    pub cell: Option<String>,
    pub village: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub google_map_url: Option<String>,
}
