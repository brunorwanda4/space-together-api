use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    STUDENT,
    TEACHER,
    ADMIN,
    SCHOOLSTAFF,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Address {
    pub country: String,
    pub province: Option<String>,
    pub district: Option<String>,
    pub sector: Option<String>,
    pub cell: Option<String>,
    pub village: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub google_map_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contact {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub alt_phone: Option<String>,
    pub whatsapp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialMedia {
    pub platform: String, // e.g. "facebook", "twitter", "instagram"
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")] // "MALE", "FEMALE"
pub enum Gender {
    MALE,
    FEMALE,
    OTHER,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct Age {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}

// format
impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Gender::MALE => "MALE",
                Gender::FEMALE => "FEMALE",
                Gender::OTHER => "OTHER",
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Language {
    English,
    French,
    Kinyarwanda,
    Kiswahili,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StudyStyle {
    Visual,
    Discussion,
    HandsOn,
    Reading,
    Writing,
    Group,
    Solo,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMethod {
    Chat,
    Sms,
    Email,
    Call,
    VideoCall,
    InPerson,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    Parent,
    Mother,
    Father,
    StepMother,
    StepFather,
    Grandmother,
    Grandfather,
    Aunt,
    Uncle,
    Brother,
    Sister,
    Cousin,
    Guardian,
    Sponsor,
    Caregiver,
    FosterParent,
    HostParent,
    Mentor,
    Teacher,
    Neighbor,
    FamilyFriend,
    LegalRepresentative,
    SocialWorker,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpecialSupport {
    /// Help paying for tuition, books, uniforms, or other costs
    Financial,

    /// Extra tutoring, remedial classes, or learning assistance
    Academic,

    /// Counseling, mentorship, or emotional support
    Emotional,

    /// Medical, physical, or mental health support
    Medical,

    /// Transportation or mobility assistance
    Mobility,

    /// Dietary or nutrition-related support
    Nutritional,

    /// Social or behavioral development support
    Social,

    /// Language learning or communication assistance
    Language,

    /// Technological support (e.g., computer access, assistive tech)
    Technical,

    /// Any other type of support not listed
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningChallenge {
    /// Struggles with understanding academic material
    NeedsTutoring,

    /// Requires help learning the language of instruction
    LanguageSupport,

    /// Has reading or writing difficulties (e.g., dyslexia)
    LiteracyDifficulty,

    /// Has attention or focus challenges (e.g., ADHD)
    AttentionDifficulty,

    /// Has a hearing impairment
    HearingImpairment,

    /// Has a visual impairment
    VisualImpairment,

    /// Has a physical or motor challenge
    PhysicalDisability,

    /// Experiences emotional or behavioral challenges
    BehavioralDifficulty,

    /// Has difficulty with math concepts (e.g., dyscalculia)
    MathDifficulty,

    /// Has cognitive or developmental learning disabilities
    LearningDisability,

    /// Needs help with organization, motivation, or study skills
    StudySkillsSupport,

    /// Any other challenge not covered above
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmploymentType {
    /// Works full-time (primary occupation)
    FullTime,

    /// Works part-time (limited hours or flexible schedule)
    PartTime,

    /// Works as a volunteer (unpaid)
    Volunteer,

    /// Works on a temporary or contract basis
    Contract,

    /// Intern or trainee position
    Internship,

    /// Self-employed or freelancer
    SelfEmployed,

    /// Unemployed or currently not working
    Unemployed,

    /// Any other type of work arrangement
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EducationLevel {
    /// No formal education
    None,

    /// Primary or elementary school
    Primary,

    /// Secondary or high school
    HighSchool,

    /// Technical or vocational education
    Vocational,

    /// College diploma or associate degree
    Diploma,

    /// Undergraduate degree (bachelor’s)
    Bachelor,

    /// Postgraduate degree (master’s)
    Master,

    /// Doctorate (PhD or equivalent)
    Doctorate,

    /// Professional or certification-based education
    Professional,

    /// Currently enrolled student
    InProgress,

    /// Any other education level not listed
    Other(String),
}

/// Represents additional certifications or trainings completed.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertificationOrTraining {
    FirstAid,
    TeachingCertificate,
    ComputerLiteracy,
    LeadershipTraining,
    SafetyTraining,
    LanguageProficiency,
    CounselingTraining,
    ChildProtection,
    ManagementTraining,
    MentorshipProgram,
    TechnicalCertification,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeachingStyle {
    /// Traditional lecture-based teaching
    Lecture,

    /// Interactive discussion-based teaching
    Discussion,

    /// Hands-on activities or experiments
    HandsOn,

    /// Project-based learning
    ProjectBased,

    /// Flipped classroom approach
    Flipped,

    /// Collaborative group work
    Collaborative,

    /// One-on-one tutoring or mentoring
    Individualized,

    /// Technology-assisted teaching (e.g., online tools)
    Digital,

    /// Any other teaching style not listed
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgeGroup {
    Age6To9,
    Age10To12,
    Age13To15,
    Age16To18,
    Grade1To3,
    Grade4To6,
    Grade7To9,
    Grade10To12,
    AdultEducation,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfessionalGoal {
    ImproveDigitalSkills,
    MentorStudents,
    ClassroomManagement,
    CurriculumDevelopment,
    AssessmentSkills,
    InclusiveEducation,
    LeadershipTraining,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyAvailability {
    pub day: Weekday,
    pub time_range: TimeRange,
}

/// Department or office where the user works
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Department {
    Administration,
    Finance,
    Library,
    IT,
    HR,
    Maintenance,
    Security,
    Cafeteria,
    Transport,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobTitle {
    Accountant,
    Secretary,
    Clerk,
    Librarian,
    SecurityGuard,
    ITSupport,
    Manager,
    Teacher,
    Counselor,
    Other(String),
}

// 🔹 Image struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub id: String,
    pub url: String,
}
