use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SubjectType {
    General,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubjectSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub class_room_id: Option<ObjectId>,
    pub class_id: Option<ObjectId>,
    pub code: String,
    pub sector_id: Option<ObjectId>,
    pub trade_id: Option<ObjectId>,
    pub subject_type: Option<SubjectType>,
    pub curriculum: Option<String>,
    pub copyright: Option<String>,
    pub learning_hours: Option<i32>,
    pub issue_date: Option<DateTime>,
    pub purpose: Option<String>,
    pub symbol: Option<String>,
    pub knowledge: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub attitude: Option<Vec<String>>,
    pub resource: Option<Vec<SubjectResource>>,
    pub competence: Option<Vec<SubjectCompetence>>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
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
