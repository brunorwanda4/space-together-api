use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        common_details::{
            Age, AgeGroup, CertificationOrTraining, CommunicationMethod, DailyAvailability,
            Department, EducationLevel, EmploymentType, Gender, Image, JobTitle, Language,
            LearningChallenge, ProfessionalGoal, SocialMedia, SpecialSupport, StudyStyle,
            TeachingStyle, UserRole,
        },
        guardian::GuardianInfo,
        subjects::subject_category::SubjectCategory,
    },
    helpers::object_id_helpers,
};

use super::common_details::Address;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    // =========================================================
    // 🔹 Identification & Authentication
    // =========================================================
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
    pub email: String,
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,

    pub role: Option<UserRole>,

    // =========================================================
    // 🔹 Profile & Media
    // =========================================================
    pub image_id: Option<String>,
    pub image: Option<String>,
    pub background_images: Option<Vec<Image>>, // GPT added for profile customization which image is url and id all are string and not optional
    pub bio: Option<String>,
    pub disable: Option<bool>,

    // =========================================================
    // 🔹 Contact & Social
    // =========================================================
    pub phone: Option<String>,
    pub address: Option<Address>,
    pub social_media: Option<Vec<SocialMedia>>,
    pub preferred_communication_method: Option<Vec<CommunicationMethod>>,

    // =========================================================
    // 🔹 Personal Information
    // =========================================================
    pub gender: Option<Gender>,
    pub age: Option<Age>,
    pub languages_spoken: Option<Vec<Language>>,
    pub hobbies_interests: Option<Vec<String>>,
    pub dream_career: Option<String>,
    pub special_skills: Option<Vec<String>>,
    pub health_or_learning_notes: Option<String>,

    // =========================================================
    // 🔹 Academic & School Relationships
    // =========================================================
    #[serde(
        serialize_with = "object_id_helpers::serialize",
        deserialize_with = "object_id_helpers::deserialize",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub current_school_id: Option<ObjectId>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub schools: Option<Vec<ObjectId>>,

    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub accessible_classes: Option<Vec<ObjectId>>,

    pub favorite_subjects_category: Option<Vec<SubjectCategory>>,
    pub preferred_study_styles: Option<Vec<StudyStyle>>,

    // =========================================================
    // 🔹 Guardian, Support & Learning Needs
    // =========================================================
    pub guardian_info: Option<Vec<GuardianInfo>>,
    pub special_support_needed: Option<Vec<SpecialSupport>>,
    pub learning_challenges: Option<Vec<LearningChallenge>>,

    // =========================================================
    // 🔹 Teaching & Employment Details
    // =========================================================
    #[serde(
        serialize_with = "object_id_helpers::serialize_opt_vec",
        deserialize_with = "object_id_helpers::deserialize_opt_vec",
        default
    )]
    pub teaching_level: Option<Vec<ObjectId>>, // relationship with trades collection

    pub employment_type: Option<EmploymentType>,
    pub teaching_start_date: Option<DateTime<Utc>>,
    pub years_of_experience: Option<DateTime<Utc>>,
    pub education_level: Option<EducationLevel>,
    pub certifications_trainings: Option<Vec<CertificationOrTraining>>,
    pub preferred_age_group: Option<AgeGroup>,
    pub professional_goals: Option<Vec<ProfessionalGoal>>,
    pub availability_schedule: Option<Vec<DailyAvailability>>, //GPT when you are creating make Default Availability schedule (e.g., "Mon–Fri 9am–5pm", "Weekends only") for all days of week monday to friday
    pub department: Option<Department>,
    pub job_title: Option<JobTitle>,
    pub teaching_style: Option<Vec<StudyStyle>>,

    // =========================================================
    // 🔹 Timestamps
    // =========================================================
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateUserDto {
    // 🔹 Basic info
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<UserRole>,

    // 🔹 Profile image
    pub image_id: Option<String>,
    pub image: Option<String>,
    pub background_images: Option<Vec<Image>>,

    // 🔹 Contact
    pub phone: Option<String>,

    // 🔹 Personal details
    pub gender: Option<Gender>,
    pub age: Option<Age>,

    // 🔹 Location
    pub address: Option<Address>,
    pub social_media: Option<Vec<SocialMedia>>,

    // 🔹 School relationships
    pub current_school_id: Option<ObjectId>,
    pub schools: Option<Vec<ObjectId>>,
    pub accessible_classes: Option<Vec<ObjectId>>,

    // 🔹 Profile
    pub bio: Option<String>,
    pub disable: Option<bool>,

    // 🔹 Academic interests
    pub favorite_subjects_category: Option<Vec<SubjectCategory>>,
    pub preferred_study_styles: Option<Vec<StudyStyle>>,
    pub languages_spoken: Option<Vec<Language>>,
    pub hobbies_interests: Option<Vec<String>>,
    pub dream_career: Option<String>,
    pub special_skills: Option<Vec<String>>,
    pub health_or_learning_notes: Option<String>,

    // 🔹 Communication preferences
    pub preferred_communication_method: Option<Vec<CommunicationMethod>>,

    // 🔹 Guardian and support info
    pub guardian_info: Option<Vec<GuardianInfo>>,
    pub special_support_needed: Option<Vec<SpecialSupport>>,
    pub learning_challenges: Option<Vec<LearningChallenge>>,

    // 🔹 Teaching-related info
    pub teaching_level: Option<Vec<ObjectId>>, // relationship with trades collection
    pub employment_type: Option<EmploymentType>,
    pub teaching_start_date: Option<DateTime<Utc>>,
    pub years_of_experience: Option<DateTime<Utc>>,
    pub education_level: Option<EducationLevel>,
    pub certifications_trainings: Option<Vec<CertificationOrTraining>>,
    pub preferred_age_group: Option<AgeGroup>,
    pub professional_goals: Option<Vec<ProfessionalGoal>>,
    pub availability_schedule: Option<Vec<DailyAvailability>>,
    pub department: Option<Department>,
    pub job_title: Option<JobTitle>,
    pub teaching_style: Option<Vec<TeachingStyle>>,

    // 🔹 Timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct UserStats {
    pub total: i64,
    pub male: i64,
    pub female: i64,
    pub other: i64,
    pub admins: i64,
    pub staff: i64,
    pub students: i64,
    pub teachers: i64,
    pub assigned_school: i64,
    pub no_school: i64,
    pub recent_30_days: i64,
}

// get class with pages
#[derive(Serialize)]
pub struct PaginatedUsers {
    pub users: Vec<User>,
    pub total: i64,
    pub total_pages: i64,
    pub current_page: i64,
}
