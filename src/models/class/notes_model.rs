use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub enum VisibilityType {
    Public,
    Private,
    Restricted,
}

#[derive(Debug, Deserialize)]
pub enum PriorityType {
    Low,
    Medium,
    High,
}

#[derive(Debug, Deserialize)]
pub struct NotesModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub reasons_id: Option<ObjectId>,
    pub teacher_id: Option<ObjectId>,
    pub comments_id: Option<Vec<ObjectId>>,
    pub attachments_id: Option<Vec<ObjectId>>,
    pub likes_id: Option<Vec<ObjectId>>,
    pub shares_id: Option<Vec<ObjectId>>,
    pub views_id: Option<Vec<ObjectId>>,
    pub created_by: String,
    pub updated_by: Option<Vec<ObjectId>>,
    pub visible_to: Option<Vec<ObjectId>>,
    pub visibility: VisibilityType,
    pub tags: Option<Vec<String>>,
    pub priority: Option<PriorityType>,
    pub due_data: Option<DateTime>,
    pub is_archive: bool,
    pub is_delete: bool,
    pub related_notes_id: Option<Vec<ObjectId>>,
    pub note_type: Option<String>,
    pub previous_version: Option<Vec<ObjectId>>,
    pub subject_id: Option<ObjectId>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
