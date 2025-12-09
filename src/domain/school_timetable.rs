use crate::{helpers::object_id_helpers, make_partial, models::default_model::default_true, utils::time_utils::is_valid_hhmm};
use chrono::{DateTime, Utc, Weekday};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};


make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBlock {
    pub title: String,
    pub start_time: String,   // "HH:MM"
    pub end_time: String,     // "HH:MM"
    #[serde(default)]
    pub description: Option<String>,
} => TimeBlockPartial
}


make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailySchoolSchedule {
    pub day: Weekday,

    #[serde(default = "default_true")]
    pub is_school_day: bool,

    pub school_start_time: String,
    pub school_end_time: String,

    pub study_start_time: String,
    pub study_end_time: String,

    #[serde(default)]
    pub breaks: Vec<TimeBlock>,

    #[serde(default)]
    pub lunch: Option<TimeBlock>,

    #[serde(default)]
    pub activities: Vec<TimeBlock>,

    #[serde(default = "default_normal_type")]
    pub special_type: DaySpecialType,
} => DailySchoolSchedulePartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DaySpecialType {
    Normal,
    HalfDay,
    Holiday,
    ExamDay,
}

fn default_normal_type() -> DaySpecialType {
    DaySpecialType::Normal
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TimetableOverrideType {
    Trade,
    Sector
}

make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimetableOverride {
    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub id: ObjectId,     //
    pub r#type: TimetableOverrideType,   // "Primary", "O-Level", etc.

    #[serde(
        serialize_with = "object_id_helpers::serialize_vec_oid",
        deserialize_with = "object_id_helpers::deserialize_vec_oid"
    )]
    pub applies_to: Vec<ObjectId>, // example: ["primary"], ["tvet_welding"]

    pub weekly_schedule: Vec<DailySchoolSchedule>,
} => TimetableOverridePartial
}


make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolEvent {
    pub event_id: String,
    pub title: String,

    #[serde(default)]
    pub description: Option<String>,

    pub start_date: DateTime<Utc>,
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,
} => SchoolEventPartial
}


make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolTimetable {
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
    pub school_id: ObjectId,

    #[serde(
        serialize_with = "object_id_helpers::serialize_oid",
        deserialize_with = "object_id_helpers::deserialize_oid"
    )]
    pub academic_year_id: ObjectId,

    pub default_weekly_schedule: Vec<DailySchoolSchedule>,

    #[serde(default)]
    pub overrides: Option<Vec<TimetableOverride>>,

    #[serde(default)]
    pub events: Option<Vec<SchoolEvent>>,

    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
} => SchoolTimetablePartial
}

impl TimeBlock {
    pub fn validate(&self) -> Result<(), String> {
        // Ensure title is not empty
        if self.title.trim().is_empty() {
            return Err("TimeBlock.title cannot be empty".into());
        }

        // Validate HH:MM format

        if !is_valid_hhmm(&self.start_time) {
                  return Err("start_time must be HH:MM format".into());
              }

              if !is_valid_hhmm(&self.end_time) {
                      return Err("end_time must be HH:MM format".into());
                  }

        Ok(())
    }
}
