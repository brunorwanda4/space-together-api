use serde::{Deserialize, Serialize};

#[derive(Debug , Clone , Deserialize , Serialize)]
pub struct ResReq {
    pub success : bool,
    pub message : String,
}