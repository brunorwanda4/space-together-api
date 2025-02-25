use serde::{Deserialize, Serialize};

use crate::libs::types::fields_types::{DateType, IdType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SubjectType {
    General,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
    pub name: String,
    pub class_room_id: Option<IdType>,
    pub class_id: Option<IdType>,
    pub code: String,
    pub sector_id: Option<IdType>,
    pub trade_id: Option<IdType>,
    pub subject_type: Option<SubjectType>,
    pub curriculum: Option<String>,
    pub copyright: Option<String>,
    pub learning_hours: Option<i32>,
    pub issue_date: Option<DateType>,
    pub purpose: Option<String>,
    pub symbol: Option<String>,
    pub knowledge: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub attitude: Option<Vec<String>>,
    pub resource: Option<Vec<SubjectResource>>,
    pub competence: Option<Vec<SubjectCompetence>>,
    pub created_at: Option<DateType>,
    pub updated_at: Option<DateType>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SubjectResourceType {
    Equipment,
    Material,
    Tools,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectResource {
    pub category: SubjectResourceType,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectCompetence {
    pub description: Option<String>,
    pub label: String,
    pub performance_criteria: Option<Vec<SubjectCompetence>>,
}
