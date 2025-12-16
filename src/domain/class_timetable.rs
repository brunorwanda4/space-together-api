use crate::{helpers::object_id_helpers, make_partial, utils::time_utils::is_valid_hhmm};
use chrono::{DateTime, Utc, Weekday};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PeriodType {
    Subject,
    Break,
    Lunch,
    Free,
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Period {
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub period_id: ObjectId,

    pub r#type: PeriodType,

    pub order: i32,
    pub title: Option<String>,
    pub description: Option<String>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub subject_id: Option<ObjectId>,

    pub start_offset: i32,                // minutes from day start
    pub duration_minutes: i32,            // positive

    pub enabled: Option<bool>,
} => PeriodPartial
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeekSchedule {
    pub day: Weekday,

    #[serde(default)]
    pub is_holiday: bool,

    /// Required when NOT holiday. Must be "HH:MM".
    pub start_on: Option<String>,

    pub periods: Vec<Period>,
} => WeekSchedulePartial
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassTimetable {
    #[serde(
        rename = "_id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub class_id: ObjectId,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub education_year_id: ObjectId,   // Reference to EducationYear
    pub term_order: i32,

    pub weekly_schedule: Vec<WeekSchedule>,
    pub disabled: Option<bool>,

    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
} => ClassTimetablePartial
}

impl Period {
    pub fn validate(&self) -> Result<(), String> {
        // Order must be non-negative
        if self.order < 0 {
            return Err("order must be non-negative".into());
        }

        // Duration must be positive
        if self.duration_minutes <= 0 {
            return Err("duration_minutes must be a positive integer".into());
        }

        // start_offset must be >= 0
        if self.start_offset < 0 {
            return Err("start_offset must be non-negative".into());
        }

        // When period type is Subject, subject_id must be present
        if matches!(self.r#type, PeriodType::Subject) && self.subject_id.is_none() {
            return Err("subject_id is required for subject periods".into());
        }

        Ok(())
    }
}

impl WeekSchedule {
    pub fn validate(&self) -> Result<(), String> {
        if !self.is_holiday {
            let Some(time) = &self.start_on else {
                return Err("start_on is required for non-holiday days".into());
            };

            if !is_valid_hhmm(time) {
                return Err("start_on must be HH:MM 24-hour format".into());
            }
        }

        for p in &self.periods {
            p.validate()?; // now works
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStructureConfig {
    pub period_type: PeriodType,
    pub duration_minutes: i32,
    pub title: Option<String>, // e.g., "Morning Break", "Lunch"
}

// Helper to create a standard school day layout
impl DayStructureConfig {
    pub fn standard_day() -> Vec<Self> {
        vec![
            // Morning Session: 2 periods
            DayStructureConfig {
                period_type: PeriodType::Subject,
                duration_minutes: 40,
                title: None,
            },
            DayStructureConfig {
                period_type: PeriodType::Subject,
                duration_minutes: 40,
                title: None,
            },
            // Break
            DayStructureConfig {
                period_type: PeriodType::Break,
                duration_minutes: 20,
                title: Some("Morning Break".into()),
            },
            // Mid Session: 2 periods
            DayStructureConfig {
                period_type: PeriodType::Subject,
                duration_minutes: 40,
                title: None,
            },
            DayStructureConfig {
                period_type: PeriodType::Subject,
                duration_minutes: 40,
                title: None,
            },
            // Lunch
            DayStructureConfig {
                period_type: PeriodType::Lunch,
                duration_minutes: 60,
                title: Some("Lunch".into()),
            },
            // Afternoon: 2 periods
            DayStructureConfig {
                period_type: PeriodType::Subject,
                duration_minutes: 40,
                title: None,
            },
            DayStructureConfig {
                period_type: PeriodType::Subject,
                duration_minutes: 40,
                title: None,
            },
        ]
    }
}
