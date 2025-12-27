use std::collections::HashMap;

use crate::{
    domain::{
        common_details::Image, main_class::MainClass, school::School, teacher::Teacher,
        trade::Trade, user::User,
    },
    helpers::object_id_helpers,
    make_partial,
};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum ClassType {
    #[default]
    Private,
    School,
    Public,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum ClassLevelType {
    #[default]
    MainClass, // e.g., "Primary 1"
    SubClass, // e.g., "Primary 1 A"
}
make_partial! {
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    #[serde(
        rename = "_id",
        // alias = "id",
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub id: Option<ObjectId>,

    pub name: String,
    pub username: String,
    pub code: Option<String>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub school_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub creator_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub class_teacher_id: Option<ObjectId>,

    #[serde(default)]
    pub r#type: ClassType, // (Private, School, Public)

    #[serde(default)]
    pub level_type: Option<ClassLevelType>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub parent_class_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub subclass_ids: Option<Vec<ObjectId>>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub main_class_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub trade_id: Option<ObjectId>,

    pub is_active: bool,

    pub image_id: Option<String>,
    pub image: Option<String>,
    pub background_images: Option<Vec<Image>>,
    pub description: Option<String>,
    pub capacity: Option<u32>,
    pub subject: Option<String>,
    pub grade_level: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    pub settings: ClassSettings,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
} => UpdateClass
}

// Add these to your existing class.rs file

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassWithOthers {
    #[serde(flatten)]
    pub class: Class,
    pub school: Option<School>,
    pub creator: Option<User>, // You'll need to define User struct
    pub class_teacher: Option<Teacher>,
    pub main_class: Option<MainClass>,
    pub trade: Option<Trade>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkClassesRequest {
    pub classes: Vec<Class>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateRequest {
    pub updates: Vec<BulkUpdateItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateItem {
    pub id: String,
    pub update: UpdateClass,
}

// ===========================
// NEW DATA STRUCTURES FOR FRONTEND
// ===========================

#[derive(Debug, serde::Serialize)]
pub struct MainClassWithSubclasses {
    pub main_class: ClassWithOthers,
    pub subclasses: Vec<Class>,
}

#[derive(Debug, serde::Serialize)]
pub struct MainClassHierarchy {
    pub main_class: ClassWithOthers,
    pub subclasses: Vec<ClassWithOthers>,
}

#[derive(Debug, serde::Serialize)]
pub struct MainClassWithSubclassCount {
    pub main_class: ClassWithOthers,
    pub subclass_count: usize,
}

// get class with pages
#[derive(Serialize)]
pub struct PaginatedClasses {
    pub classes: Vec<Class>,
    pub total: i64,
    pub total_pages: i64,
    pub current_page: i64,
}

#[derive(Serialize)]
pub struct PaginatedClassesWithOthers {
    pub classes: Vec<ClassWithOthers>,
    pub total: i64,
    pub total_pages: i64,
    pub current_page: i64,
}

// ================= Class settings ==================================

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum StudentVisibility {
    #[default]
    All,
    Limited,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentPermissions {
    pub can_chat: Option<bool>,
    pub can_upload_homework: Option<bool>,
    pub can_comment: Option<bool>,
    pub can_view_all_students: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceRules {
    pub late_after_minutes: Option<u32>,
    pub required_attendance_percentage: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassworkRules {
    pub allow_resubmission: Option<bool>,
    pub max_late_days: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassStudentSettings {
    pub auto_enroll_subclasses: Option<bool>,
    pub student_visibility: Option<StudentVisibility>,
    pub permissions: Option<StudentPermissions>,
    pub attendance_rules: Option<AttendanceRules>,
    pub classwork_rules: Option<ClassworkRules>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeacherPermissions {
    pub can_edit_marks: Option<bool>,
    pub can_take_attendance: Option<bool>,
    pub can_remove_students: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassTeacherSettings {
    pub permissions: Option<TeacherPermissions>,
    pub visibility: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllowedActions {
    pub can_edit_class_info: Option<bool>,
    pub can_add_students: Option<bool>,
    pub can_remove_students: Option<bool>,
    pub can_manage_subjects: Option<bool>,
    pub can_manage_timetable: Option<bool>,
    pub can_approve_requests: Option<bool>,
    pub can_assign_roles: Option<bool>,
    pub can_send_parent_notifications: Option<bool>,
    pub can_add_teachers: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecuritySettings {
    pub require_two_person_approval_for_results: Option<bool>,
    pub log_all_teacher_changes: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassClassTeacherSettings {
    pub allowed_actions: Option<AllowedActions>,
    pub security: Option<SecuritySettings>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimetablePeriod {
    pub period: Option<u32>,
    pub subject: Option<String>,
    pub teacher_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreakTime {
    pub start: Option<String>,
    pub end: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClashPrevention {
    pub prevent_double_teacher_booking: Option<bool>,
    pub prevent_duplicate_subject_same_day: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassTimetableSettings {
    pub period_length_minutes: Option<u32>,
    pub periods_per_day: Option<u32>,
    pub weekly_timetable: Option<HashMap<String, Vec<TimetablePeriod>>>,
    pub break_times: Option<Vec<BreakTime>>,
    pub clash_prevention: Option<ClashPrevention>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassSettings {
    pub students: Option<ClassStudentSettings>,
    pub teachers: Option<ClassTeacherSettings>,
    pub class_teacher: Option<ClassClassTeacherSettings>,
    pub timetable: Option<ClassTimetableSettings>,
}

impl ClassSettings {
    pub fn default() -> Self {
        Self {
            students: Some(ClassStudentSettings {
                auto_enroll_subclasses: Some(false),
                student_visibility: Some(StudentVisibility::All),
                permissions: Some(StudentPermissions {
                    can_chat: Some(true),
                    can_upload_homework: Some(true),
                    can_comment: Some(true),
                    can_view_all_students: Some(false),
                }),
                attendance_rules: Some(AttendanceRules {
                    late_after_minutes: Some(10),
                    required_attendance_percentage: Some(75.0),
                }),
                classwork_rules: Some(ClassworkRules {
                    allow_resubmission: Some(true),
                    max_late_days: Some("3".to_string()),
                }),
            }),

            teachers: Some(ClassTeacherSettings {
                permissions: Some(TeacherPermissions {
                    can_edit_marks: Some(true),
                    can_take_attendance: Some(true),
                    can_remove_students: Some(false),
                }),
                visibility: Some(true),
            }),

            class_teacher: Some(ClassClassTeacherSettings {
                allowed_actions: Some(AllowedActions {
                    can_edit_class_info: Some(true),
                    can_add_students: Some(true),
                    can_remove_students: Some(true),
                    can_manage_subjects: Some(true),
                    can_manage_timetable: Some(true),
                    can_approve_requests: Some(true),
                    can_assign_roles: Some(true),
                    can_send_parent_notifications: Some(true),
                    can_add_teachers: Some(true),
                }),
                security: Some(SecuritySettings {
                    require_two_person_approval_for_results: Some(false),
                    log_all_teacher_changes: Some(true),
                }),
            }),

            timetable: Some(ClassTimetableSettings {
                period_length_minutes: Some(45),
                periods_per_day: Some(8),
                weekly_timetable: Some(HashMap::new()),
                break_times: Some(vec![
                    BreakTime {
                        start: Some("10:30".into()),
                        end: Some("10:45".into()),
                        label: Some("Morning Break".into()),
                    },
                    BreakTime {
                        start: Some("13:00".into()),
                        end: Some("14:00".into()),
                        label: Some("Lunch Break".into()),
                    },
                ]),
                clash_prevention: Some(ClashPrevention {
                    prevent_double_teacher_booking: Some(true),
                    prevent_duplicate_subject_same_day: Some(true),
                }),
            }),
        }
    }
}
