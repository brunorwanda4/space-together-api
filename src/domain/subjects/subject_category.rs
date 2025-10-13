use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for SubjectCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectCategory::Science => write!(f, "Science"),
            SubjectCategory::Technology => write!(f, "Technology"),
            SubjectCategory::Engineering => write!(f, "Engineering"),
            SubjectCategory::Mathematics => write!(f, "Mathematics"),
            SubjectCategory::Language => write!(f, "Language"),
            SubjectCategory::SocialScience => write!(f, "Social Science"),
            SubjectCategory::Arts => write!(f, "Arts"),
            SubjectCategory::TVET => write!(f, "TVET"),
            SubjectCategory::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectTypeFor {
    MainSubject,
    ClassSubject,
}
