use crate::domain::common_details::Relationship;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuardianInfo {
    pub name: Option<String>,

    pub phone: Option<String>,

    pub email: Option<String>,

    pub relationship: Option<Relationship>,
}
