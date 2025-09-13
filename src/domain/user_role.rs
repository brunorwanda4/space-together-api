use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    STUDENT,
    TEACHER,
    ADMIN,
    SCHOOLSTAFF,
}
