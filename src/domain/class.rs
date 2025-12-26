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

    /// 👇 NEW: Is this a MainClass or a SubClass?
    #[serde(default)]
    pub level_type: Option<ClassLevelType>,

    /// 👇 NEW: If SubClass, reference to its main class (e.g., "Primary 1")
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub parent_class_id: Option<ObjectId>,

    /// 👇 OPTIONAL: If this is a main class, list all its subclasses
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

    pub settings: Option<ClassSettings>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StudentVisibility {
    #[default]
    All,
    Limited,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct StudentPermissions {
    pub can_chat: bool,
    pub can_upload_homework: bool,
    pub can_comment: bool,
    pub can_view_all_students: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AttendanceRules {
    pub late_after_minutes: u32,
    pub required_attendance_percentage: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClassworkRules {
    pub allow_resubmission: bool,
    pub max_late_days: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClassStudentSettings {
    pub auto_enroll_subclasses: bool,
    pub student_visibility: StudentVisibility,
    pub permissions: StudentPermissions,
    pub attendance_rules: AttendanceRules,
    pub classwork_rules: ClassworkRules,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct TeacherPermissions {
    pub can_edit_marks: bool,
    pub can_take_attendance: bool,
    pub can_remove_students: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClassTeacherSettings {
    pub permissions: TeacherPermissions,
    pub visibility: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AllowedActions {
    pub can_edit_class_info: bool,
    pub can_add_students: bool,
    pub can_remove_students: bool,
    pub can_manage_subjects: bool,
    pub can_manage_timetable: bool,
    pub can_approve_requests: bool,
    pub can_assign_roles: bool,
    pub can_send_parent_notifications: bool,
    pub can_add_teachers: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct SecuritySettings {
    pub require_two_person_approval_for_results: bool,
    pub log_all_teacher_changes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClassClassTeacherSettings {
    pub allowed_actions: AllowedActions,
    pub security: SecuritySettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct TimetablePeriod {
    pub period: u32,
    pub subject: String,
    pub teacher_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct BreakTime {
    pub start: String,
    pub end: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClashPrevention {
    pub prevent_double_teacher_booking: bool,
    pub prevent_duplicate_subject_same_day: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClassTimetableSettings {
    pub period_length_minutes: u32,
    pub periods_per_day: u32,

    /// key = weekday (e.g. "monday")
    pub weekly_timetable: HashMap<String, Vec<TimetablePeriod>>,

    pub break_times: Vec<BreakTime>,
    pub clash_prevention: ClashPrevention,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ClassSettings {
    pub students: ClassStudentSettings,
    pub teachers: ClassTeacherSettings,
    pub class_teacher: ClassClassTeacherSettings,
    pub timetable: ClassTimetableSettings,
}

impl ClassSettings {
    pub fn default() -> Self {
        ClassSettings {
            students: ClassStudentSettings {
                auto_enroll_subclasses: false,
                student_visibility: StudentVisibility::All,
                permissions: StudentPermissions {
                    can_chat: true,
                    can_upload_homework: true,
                    can_comment: true,
                    can_view_all_students: false,
                },
                attendance_rules: AttendanceRules {
                    late_after_minutes: 10,
                    required_attendance_percentage: 75.0,
                },
                classwork_rules: ClassworkRules {
                    allow_resubmission: true,
                    max_late_days: "3".to_string(),
                },
            },

            teachers: ClassTeacherSettings {
                permissions: TeacherPermissions {
                    can_edit_marks: true,
                    can_take_attendance: true,
                    can_remove_students: false,
                },
                visibility: true,
            },

            class_teacher: ClassClassTeacherSettings {
                allowed_actions: AllowedActions {
                    can_edit_class_info: true,
                    can_add_students: true,
                    can_remove_students: true,
                    can_manage_subjects: true,
                    can_manage_timetable: true,
                    can_approve_requests: true,
                    can_assign_roles: true,
                    can_send_parent_notifications: true,
                    can_add_teachers: true,
                },
                security: SecuritySettings {
                    require_two_person_approval_for_results: false,
                    log_all_teacher_changes: true,
                },
            },

            timetable: ClassTimetableSettings {
                period_length_minutes: 45,
                periods_per_day: 8,
                weekly_timetable: std::collections::HashMap::new(),
                break_times: vec![
                    BreakTime {
                        start: "10:30".to_string(),
                        end: "10:45".to_string(),
                        label: "Morning Break".to_string(),
                    },
                    BreakTime {
                        start: "13:00".to_string(),
                        end: "14:00".to_string(),
                        label: "Lunch Break".to_string(),
                    },
                ],
                clash_prevention: ClashPrevention {
                    prevent_double_teacher_booking: true,
                    prevent_duplicate_subject_same_day: true,
                },
            },
        }
    }
}
