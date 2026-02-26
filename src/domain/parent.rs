use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{domain::common_details::Gender, helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ParentStatus {
    #[default]
    Active,
    Inactive,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Parent {
        #[serde(
            rename = "_id",
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
        pub user_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub school_id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize_opt_vec",
            deserialize_with = "object_id_helpers::deserialize_opt_vec",
            default
        )]
        pub student_ids: Option<Vec<ObjectId>>,

        pub name: String,
        pub email: String,
        pub phone: Option<String>,
        pub gender: Option<Gender>,
        pub image: Option<String>,
        pub image_id: Option<String>,

        pub relationship: Option<String>, // e.g., "Father", "Mother", "Guardian"
        pub occupation: Option<String>,
        pub national_id: Option<String>,

        #[serde(default)]
        pub status: ParentStatus,

        #[serde(default)]
        pub is_active: bool,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        #[serde(default = "Utc::now")]
        pub updated_at: DateTime<Utc>,
    } => ParentPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParentWithRelations {
    #[serde(flatten)]
    pub parent: Parent,

    pub user: Option<crate::domain::user::User>,
    pub school: Option<crate::domain::school::School>,
    pub students: Option<Vec<crate::domain::student::Student>>,
}

// Dashboard response DTOs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParentDashboard {
    pub total_children: i64,
    pub latest_announcements: Vec<crate::domain::announcement::AnnouncementWithRelations>,
    pub children_summary: Vec<ChildSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChildSummary {
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub student_id: Option<ObjectId>,
    pub student_name: String,
    pub class_name: Option<String>,
    pub attendance_percentage: f64,
    pub current_term_gpa: f64,
    pub outstanding_fees: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceSummary {
    pub present_count: i64,
    pub absent_count: i64,
    pub late_count: i64,
    pub excused_count: i64,
    pub total_days: i64,
    pub attendance_percentage: f64,
    pub recent_records: Vec<AttendanceRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceRecord {
    pub date: DateTime<Utc>,
    pub status: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentResults {
    pub term_gpa: f64,
    pub rank: Option<i32>,
    pub total_students: Option<i32>,
    pub grade: String,
    pub subject_results: Vec<SubjectGrade>,
    pub teacher_remarks: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectGrade {
    pub subject_name: String,
    pub score: f64,
    pub max_score: f64,
    pub percentage: f64,
    pub grade: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinanceSummary {
    pub total_fee_required: f64,
    pub amount_paid: f64,
    pub outstanding_balance: f64,
    pub payment_history: Vec<PaymentRecord>,
    pub installments: Vec<InstallmentInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentRecord {
    pub date: DateTime<Utc>,
    pub amount: f64,
    pub payment_method: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallmentInfo {
    pub due_date: DateTime<Utc>,
    pub amount: f64,
    pub status: String,
}
