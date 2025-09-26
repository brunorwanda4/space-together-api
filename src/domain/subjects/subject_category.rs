use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectCategory {
    Science,
    Technology,
    Engineering,
    Mathematics,
    Language,
    SocialScience,
    Arts,
    TVET,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectTypeFor {
    MainSubject,
    ClassSubject,
}
