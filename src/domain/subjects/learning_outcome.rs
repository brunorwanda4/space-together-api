use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{
    domain::subjects::{
        subject_category::SubjectTypeFor, subject_competency_block::SubjectCompetencyBlock,
        subject_topic::SubjectTopicWithOthers,
    },
    helpers::object_id_helpers,
};

/// Main Learning Outcome struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningOutcome {
    #[serde(
        rename = "_id",
        alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub subject_id: Option<ObjectId>, // Reference to MainSubject

    pub title: String,
    pub description: Option<String>,
    pub order: i32,
    pub estimated_hours: Option<i32>,
    pub key_competencies: SubjectCompetencyBlock,
    pub assessment_criteria: Vec<String>,
    pub role: SubjectTypeFor,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub prerequisites: Option<Vec<ObjectId>>, // Reference other outcomes

    pub is_mandatory: Option<bool>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub created_by: Option<ObjectId>,

    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Update struct for LearningOutcome
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateLearningOutcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<ObjectId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_competencies: Option<SubjectCompetencyBlock>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment_criteria: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<SubjectTypeFor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<Vec<ObjectId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mandatory: Option<bool>,
}

/// LearningOutcome with nested topics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningOutcomeWithOthers {
    #[serde(flatten)]
    pub learning_out: LearningOutcome,

    #[serde(default, deserialize_with = "deserialize_topics_option")]
    pub topics: Option<Vec<SubjectTopicWithOthers>>,
}

/// Custom deserializer for topics
fn deserialize_topics_option<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<SubjectTopicWithOthers>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Array(arr) => {
            let topics: Vec<SubjectTopicWithOthers> = arr
                .into_iter()
                .map(|v| serde_json::from_value(v).map_err(serde::de::Error::custom))
                .collect::<Result<_, _>>()?;
            Ok(Some(topics))
        }
        Value::Object(map) => {
            let topic: SubjectTopicWithOthers =
                serde_json::from_value(Value::Object(map)).map_err(serde::de::Error::custom)?;
            Ok(Some(vec![topic]))
        }
        Value::Null => Ok(None),
        _ => Err(serde::de::Error::custom("invalid type for topics")),
    }
}
