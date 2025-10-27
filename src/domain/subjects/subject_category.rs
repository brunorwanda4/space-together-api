use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
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

impl From<SubjectCategory> for String {
    fn from(s: SubjectCategory) -> String {
        match s {
            SubjectCategory::Science => "Science".to_string(),
            SubjectCategory::Technology => "Technology".to_string(),
            SubjectCategory::Engineering => "Engineering".to_string(),
            SubjectCategory::Mathematics => "Mathematics".to_string(),
            SubjectCategory::Language => "Language".to_string(),
            SubjectCategory::SocialScience => "SocialScience".to_string(),
            SubjectCategory::Arts => "Arts".to_string(),
            SubjectCategory::TVET => "TVET".to_string(),
            SubjectCategory::Other(x) => x,
        }
    }
}

impl From<String> for SubjectCategory {
    fn from(s: String) -> Self {
        // keep original for Other(...) but match on a normalized form
        let raw = s.clone();
        let normalized: String = s
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();

        match normalized.as_str() {
            "science" => SubjectCategory::Science,
            "technology" => SubjectCategory::Technology,
            "engineering" => SubjectCategory::Engineering,
            "mathematics" | "math" => SubjectCategory::Mathematics,
            "language" => SubjectCategory::Language,
            "socialscience" | "socialsci" | "socialscience" => SubjectCategory::SocialScience,
            "arts" => SubjectCategory::Arts,
            "tvet" => SubjectCategory::TVET,
            _ => SubjectCategory::Other(raw),
        }
    }
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
