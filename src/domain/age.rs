use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Age {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}
