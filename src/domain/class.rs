use crate::{
    domain::{
        common_details::Image, main_class::MainClass, school::School, teacher::Teacher,
        trade::Trade, user::User,
    },
    helpers::object_id_helpers,
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    #[serde(
        rename = "_id",
        alias = "id",
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

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct UpdateClass {
    pub name: Option<String>,
    pub username: Option<String>,
    pub code: Option<Option<String>>, // Allows setting to None
    pub image_id: Option<String>,
    pub image: Option<String>,
    pub background_images: Option<Vec<Image>>,
    pub school_id: Option<Option<ObjectId>>,
    pub r#type: Option<ClassType>,
    pub is_active: Option<bool>,
    pub description: Option<Option<String>>,
    pub capacity: Option<u32>,
    pub subject: Option<Option<String>>,
    pub grade_level: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

// Add these to your existing class.rs file

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassWithSchool {
    #[serde(flatten)]
    pub class: Class,
    pub school: Option<School>, // You'll need to define School struct
}

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
pub struct ClassStatistics {
    pub total_classes: usize,
    pub main_classes: usize,
    pub subclasses: usize,
    pub active_classes: usize,
    pub inactive_classes: usize,
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
