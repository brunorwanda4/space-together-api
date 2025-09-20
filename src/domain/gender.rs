use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")] // "MALE", "FEMALE"
pub enum Gender {
    MALE,
    FEMALE,
    OTHER,
}
