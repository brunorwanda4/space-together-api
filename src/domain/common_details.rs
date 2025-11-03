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
    pub street: Option<String>,
    pub city: Option<String>,
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

// ------------------ enums with Other(String) converted to/from String ------------------ //

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum StudyStyle {
    Visual,
    Discussion,
    HandsOn,
    Reading,
    Writing,
    Group,
    Solo,
    ProjectBased,
    Digital,
    Other(String),
}

impl From<StudyStyle> for String {
    fn from(s: StudyStyle) -> String {
        match s {
            StudyStyle::Visual => "Visual".to_string(),
            StudyStyle::Discussion => "Discussion".to_string(),
            StudyStyle::HandsOn => "HandsOn".to_string(),
            StudyStyle::Reading => "Reading".to_string(),
            StudyStyle::Writing => "Writing".to_string(),
            StudyStyle::Group => "Group".to_string(),
            StudyStyle::Solo => "Solo".to_string(),
            StudyStyle::ProjectBased => "ProjectBased".to_string(),
            StudyStyle::Digital => "Digital".to_string(),
            StudyStyle::Other(x) => x,
        }
    }
}

impl From<String> for StudyStyle {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Visual" => StudyStyle::Visual,
            "Discussion" => StudyStyle::Discussion,
            "HandsOn" => StudyStyle::HandsOn,
            "Reading" => StudyStyle::Reading,
            "Writing" => StudyStyle::Writing,
            "Group" => StudyStyle::Group,
            "Solo" => StudyStyle::Solo,
            "ProjectBased" => StudyStyle::ProjectBased,
            "Digital" => StudyStyle::Digital,
            _ => StudyStyle::Other(raw),
        }
    }
}

impl fmt::Display for StudyStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StudyStyle::Visual => write!(f, "Visual"),
            StudyStyle::Discussion => write!(f, "Discussion"),
            StudyStyle::HandsOn => write!(f, "HandsOn"),
            StudyStyle::Reading => write!(f, "Reading"),
            StudyStyle::Writing => write!(f, "Writing"),
            StudyStyle::Group => write!(f, "Group"),
            StudyStyle::Solo => write!(f, "Solo"),
            StudyStyle::ProjectBased => write!(f, "ProjectBased"),
            StudyStyle::Digital => write!(f, "Digital"),
            StudyStyle::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum CommunicationMethod {
    Chat,
    Sms,
    Email,
    Call,
    VideoCall,
    InPerson,
    Other(String),
}

impl From<CommunicationMethod> for String {
    fn from(m: CommunicationMethod) -> String {
        match m {
            CommunicationMethod::Chat => "Chat".to_string(),
            CommunicationMethod::Sms => "Sms".to_string(),
            CommunicationMethod::Email => "Email".to_string(),
            CommunicationMethod::Call => "Call".to_string(),
            CommunicationMethod::VideoCall => "VideoCall".to_string(),
            CommunicationMethod::InPerson => "InPerson".to_string(),
            CommunicationMethod::Other(x) => x,
        }
    }
}

impl From<String> for CommunicationMethod {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Chat" => CommunicationMethod::Chat,
            "Sms" => CommunicationMethod::Sms,
            "Email" => CommunicationMethod::Email,
            "Call" => CommunicationMethod::Call,
            "VideoCall" => CommunicationMethod::VideoCall,
            "InPerson" => CommunicationMethod::InPerson,
            _ => CommunicationMethod::Other(raw),
        }
    }
}

impl fmt::Display for CommunicationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommunicationMethod::Chat => write!(f, "Chat"),
            CommunicationMethod::Sms => write!(f, "Sms"),
            CommunicationMethod::Email => write!(f, "Email"),
            CommunicationMethod::Call => write!(f, "Call"),
            CommunicationMethod::VideoCall => write!(f, "VideoCall"),
            CommunicationMethod::InPerson => write!(f, "InPerson"),
            CommunicationMethod::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
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

impl From<Relationship> for String {
    fn from(r: Relationship) -> String {
        match r {
            Relationship::Parent => "Parent".to_string(),
            Relationship::Mother => "Mother".to_string(),
            Relationship::Father => "Father".to_string(),
            Relationship::StepMother => "StepMother".to_string(),
            Relationship::StepFather => "StepFather".to_string(),
            Relationship::Grandmother => "Grandmother".to_string(),
            Relationship::Grandfather => "Grandfather".to_string(),
            Relationship::Aunt => "Aunt".to_string(),
            Relationship::Uncle => "Uncle".to_string(),
            Relationship::Brother => "Brother".to_string(),
            Relationship::Sister => "Sister".to_string(),
            Relationship::Cousin => "Cousin".to_string(),
            Relationship::Guardian => "Guardian".to_string(),
            Relationship::Sponsor => "Sponsor".to_string(),
            Relationship::Caregiver => "Caregiver".to_string(),
            Relationship::FosterParent => "FosterParent".to_string(),
            Relationship::HostParent => "HostParent".to_string(),
            Relationship::Mentor => "Mentor".to_string(),
            Relationship::Teacher => "Teacher".to_string(),
            Relationship::Neighbor => "Neighbor".to_string(),
            Relationship::FamilyFriend => "FamilyFriend".to_string(),
            Relationship::LegalRepresentative => "LegalRepresentative".to_string(),
            Relationship::SocialWorker => "SocialWorker".to_string(),
            Relationship::Other(x) => x,
        }
    }
}

impl From<String> for Relationship {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Parent" => Relationship::Parent,
            "Mother" => Relationship::Mother,
            "Father" => Relationship::Father,
            "StepMother" => Relationship::StepMother,
            "StepFather" => Relationship::StepFather,
            "Grandmother" => Relationship::Grandmother,
            "Grandfather" => Relationship::Grandfather,
            "Aunt" => Relationship::Aunt,
            "Uncle" => Relationship::Uncle,
            "Brother" => Relationship::Brother,
            "Sister" => Relationship::Sister,
            "Cousin" => Relationship::Cousin,
            "Guardian" => Relationship::Guardian,
            "Sponsor" => Relationship::Sponsor,
            "Caregiver" => Relationship::Caregiver,
            "FosterParent" => Relationship::FosterParent,
            "HostParent" => Relationship::HostParent,
            "Mentor" => Relationship::Mentor,
            "Teacher" => Relationship::Teacher,
            "Neighbor" => Relationship::Neighbor,
            "FamilyFriend" => Relationship::FamilyFriend,
            "LegalRepresentative" => Relationship::LegalRepresentative,
            "SocialWorker" => Relationship::SocialWorker,
            _ => Relationship::Other(raw),
        }
    }
}

impl fmt::Display for Relationship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relationship::Parent => write!(f, "Parent"),
            Relationship::Mother => write!(f, "Mother"),
            Relationship::Father => write!(f, "Father"),
            Relationship::StepMother => write!(f, "StepMother"),
            Relationship::StepFather => write!(f, "StepFather"),
            Relationship::Grandmother => write!(f, "Grandmother"),
            Relationship::Grandfather => write!(f, "Grandfather"),
            Relationship::Aunt => write!(f, "Aunt"),
            Relationship::Uncle => write!(f, "Uncle"),
            Relationship::Brother => write!(f, "Brother"),
            Relationship::Sister => write!(f, "Sister"),
            Relationship::Cousin => write!(f, "Cousin"),
            Relationship::Guardian => write!(f, "Guardian"),
            Relationship::Sponsor => write!(f, "Sponsor"),
            Relationship::Caregiver => write!(f, "Caregiver"),
            Relationship::FosterParent => write!(f, "FosterParent"),
            Relationship::HostParent => write!(f, "HostParent"),
            Relationship::Mentor => write!(f, "Mentor"),
            Relationship::Teacher => write!(f, "Teacher"),
            Relationship::Neighbor => write!(f, "Neighbor"),
            Relationship::FamilyFriend => write!(f, "FamilyFriend"),
            Relationship::LegalRepresentative => write!(f, "LegalRepresentative"),
            Relationship::SocialWorker => write!(f, "SocialWorker"),
            Relationship::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum SpecialSupport {
    Financial,
    Academic,
    Emotional,
    Medical,
    Mobility,
    Nutritional,
    Social,
    Language,
    Technical,
    Other(String),
}

impl From<SpecialSupport> for String {
    fn from(s: SpecialSupport) -> String {
        match s {
            SpecialSupport::Financial => "Financial".to_string(),
            SpecialSupport::Academic => "Academic".to_string(),
            SpecialSupport::Emotional => "Emotional".to_string(),
            SpecialSupport::Medical => "Medical".to_string(),
            SpecialSupport::Mobility => "Mobility".to_string(),
            SpecialSupport::Nutritional => "Nutritional".to_string(),
            SpecialSupport::Social => "Social".to_string(),
            SpecialSupport::Language => "Language".to_string(),
            SpecialSupport::Technical => "Technical".to_string(),
            SpecialSupport::Other(x) => x,
        }
    }
}

impl From<String> for SpecialSupport {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Financial" => SpecialSupport::Financial,
            "Academic" => SpecialSupport::Academic,
            "Emotional" => SpecialSupport::Emotional,
            "Medical" => SpecialSupport::Medical,
            "Mobility" => SpecialSupport::Mobility,
            "Nutritional" => SpecialSupport::Nutritional,
            "Social" => SpecialSupport::Social,
            "Language" => SpecialSupport::Language,
            "Technical" => SpecialSupport::Technical,
            _ => SpecialSupport::Other(raw),
        }
    }
}

impl fmt::Display for SpecialSupport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecialSupport::Financial => write!(f, "Financial"),
            SpecialSupport::Academic => write!(f, "Academic"),
            SpecialSupport::Emotional => write!(f, "Emotional"),
            SpecialSupport::Medical => write!(f, "Medical"),
            SpecialSupport::Mobility => write!(f, "Mobility"),
            SpecialSupport::Nutritional => write!(f, "Nutritional"),
            SpecialSupport::Social => write!(f, "Social"),
            SpecialSupport::Language => write!(f, "Language"),
            SpecialSupport::Technical => write!(f, "Technical"),
            SpecialSupport::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum LearningChallenge {
    NeedsTutoring,
    LanguageSupport,
    LiteracyDifficulty,
    AttentionDifficulty,
    HearingImpairment,
    VisualImpairment,
    PhysicalDisability,
    BehavioralDifficulty,
    MathDifficulty,
    LearningDisability,
    StudySkillsSupport,
    Other(String),
}

impl From<LearningChallenge> for String {
    fn from(l: LearningChallenge) -> String {
        match l {
            LearningChallenge::NeedsTutoring => "NeedsTutoring".to_string(),
            LearningChallenge::LanguageSupport => "LanguageSupport".to_string(),
            LearningChallenge::LiteracyDifficulty => "LiteracyDifficulty".to_string(),
            LearningChallenge::AttentionDifficulty => "AttentionDifficulty".to_string(),
            LearningChallenge::HearingImpairment => "HearingImpairment".to_string(),
            LearningChallenge::VisualImpairment => "VisualImpairment".to_string(),
            LearningChallenge::PhysicalDisability => "PhysicalDisability".to_string(),
            LearningChallenge::BehavioralDifficulty => "BehavioralDifficulty".to_string(),
            LearningChallenge::MathDifficulty => "MathDifficulty".to_string(),
            LearningChallenge::LearningDisability => "LearningDisability".to_string(),
            LearningChallenge::StudySkillsSupport => "StudySkillsSupport".to_string(),
            LearningChallenge::Other(x) => x,
        }
    }
}

impl From<String> for LearningChallenge {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "NeedsTutoring" => LearningChallenge::NeedsTutoring,
            "LanguageSupport" => LearningChallenge::LanguageSupport,
            "LiteracyDifficulty" => LearningChallenge::LiteracyDifficulty,
            "AttentionDifficulty" => LearningChallenge::AttentionDifficulty,
            "HearingImpairment" => LearningChallenge::HearingImpairment,
            "VisualImpairment" => LearningChallenge::VisualImpairment,
            "PhysicalDisability" => LearningChallenge::PhysicalDisability,
            "BehavioralDifficulty" => LearningChallenge::BehavioralDifficulty,
            "MathDifficulty" => LearningChallenge::MathDifficulty,
            "LearningDisability" => LearningChallenge::LearningDisability,
            "StudySkillsSupport" => LearningChallenge::StudySkillsSupport,
            _ => LearningChallenge::Other(raw),
        }
    }
}

impl fmt::Display for LearningChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LearningChallenge::NeedsTutoring => write!(f, "NeedsTutoring"),
            LearningChallenge::LanguageSupport => write!(f, "LanguageSupport"),
            LearningChallenge::LiteracyDifficulty => write!(f, "LiteracyDifficulty"),
            LearningChallenge::AttentionDifficulty => write!(f, "AttentionDifficulty"),
            LearningChallenge::HearingImpairment => write!(f, "HearingImpairment"),
            LearningChallenge::VisualImpairment => write!(f, "VisualImpairment"),
            LearningChallenge::PhysicalDisability => write!(f, "PhysicalDisability"),
            LearningChallenge::BehavioralDifficulty => write!(f, "BehavioralDifficulty"),
            LearningChallenge::MathDifficulty => write!(f, "MathDifficulty"),
            LearningChallenge::LearningDisability => write!(f, "LearningDisability"),
            LearningChallenge::StudySkillsSupport => write!(f, "StudySkillsSupport"),
            LearningChallenge::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Volunteer,
    Contract,
    Internship,
    SelfEmployed,
    Unemployed,
    Other(String),
}

impl From<EmploymentType> for String {
    fn from(e: EmploymentType) -> String {
        match e {
            EmploymentType::FullTime => "FullTime".to_string(),
            EmploymentType::PartTime => "PartTime".to_string(),
            EmploymentType::Volunteer => "Volunteer".to_string(),
            EmploymentType::Contract => "Contract".to_string(),
            EmploymentType::Internship => "Internship".to_string(),
            EmploymentType::SelfEmployed => "SelfEmployed".to_string(),
            EmploymentType::Unemployed => "Unemployed".to_string(),
            EmploymentType::Other(x) => x,
        }
    }
}

impl From<String> for EmploymentType {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "FullTime" => EmploymentType::FullTime,
            "PartTime" => EmploymentType::PartTime,
            "Volunteer" => EmploymentType::Volunteer,
            "Contract" => EmploymentType::Contract,
            "Internship" => EmploymentType::Internship,
            "SelfEmployed" => EmploymentType::SelfEmployed,
            "Unemployed" => EmploymentType::Unemployed,
            _ => EmploymentType::Other(raw),
        }
    }
}

impl fmt::Display for EmploymentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmploymentType::FullTime => write!(f, "FullTime"),
            EmploymentType::PartTime => write!(f, "PartTime"),
            EmploymentType::Volunteer => write!(f, "Volunteer"),
            EmploymentType::Contract => write!(f, "Contract"),
            EmploymentType::Internship => write!(f, "Internship"),
            EmploymentType::SelfEmployed => write!(f, "SelfEmployed"),
            EmploymentType::Unemployed => write!(f, "Unemployed"),
            EmploymentType::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum EducationLevel {
    None,
    Primary,
    HighSchool,
    Vocational,
    Diploma,
    Bachelor,
    Master,
    Doctorate,
    Professional,
    InProgress,
    Other(String),
}

impl From<EducationLevel> for String {
    fn from(e: EducationLevel) -> String {
        match e {
            EducationLevel::None => "None".to_string(),
            EducationLevel::Primary => "Primary".to_string(),
            EducationLevel::HighSchool => "HighSchool".to_string(),
            EducationLevel::Vocational => "Vocational".to_string(),
            EducationLevel::Diploma => "Diploma".to_string(),
            EducationLevel::Bachelor => "Bachelor".to_string(),
            EducationLevel::Master => "Master".to_string(),
            EducationLevel::Doctorate => "Doctorate".to_string(),
            EducationLevel::Professional => "Professional".to_string(),
            EducationLevel::InProgress => "InProgress".to_string(),
            EducationLevel::Other(x) => x,
        }
    }
}

impl From<String> for EducationLevel {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "None" => EducationLevel::None,
            "Primary" => EducationLevel::Primary,
            "HighSchool" => EducationLevel::HighSchool,
            "Vocational" => EducationLevel::Vocational,
            "Diploma" => EducationLevel::Diploma,
            "Bachelor" => EducationLevel::Bachelor,
            "Master" => EducationLevel::Master,
            "Doctorate" => EducationLevel::Doctorate,
            "Professional" => EducationLevel::Professional,
            "InProgress" => EducationLevel::InProgress,
            _ => EducationLevel::Other(raw),
        }
    }
}

impl fmt::Display for EducationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EducationLevel::None => write!(f, "None"),
            EducationLevel::Primary => write!(f, "Primary"),
            EducationLevel::HighSchool => write!(f, "HighSchool"),
            EducationLevel::Vocational => write!(f, "Vocational"),
            EducationLevel::Diploma => write!(f, "Diploma"),
            EducationLevel::Bachelor => write!(f, "Bachelor"),
            EducationLevel::Master => write!(f, "Master"),
            EducationLevel::Doctorate => write!(f, "Doctorate"),
            EducationLevel::Professional => write!(f, "Professional"),
            EducationLevel::InProgress => write!(f, "InProgress"),
            EducationLevel::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
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

impl From<CertificationOrTraining> for String {
    fn from(c: CertificationOrTraining) -> String {
        match c {
            CertificationOrTraining::FirstAid => "FirstAid".to_string(),
            CertificationOrTraining::TeachingCertificate => "TeachingCertificate".to_string(),
            CertificationOrTraining::ComputerLiteracy => "ComputerLiteracy".to_string(),
            CertificationOrTraining::LeadershipTraining => "LeadershipTraining".to_string(),
            CertificationOrTraining::SafetyTraining => "SafetyTraining".to_string(),
            CertificationOrTraining::LanguageProficiency => "LanguageProficiency".to_string(),
            CertificationOrTraining::CounselingTraining => "CounselingTraining".to_string(),
            CertificationOrTraining::ChildProtection => "ChildProtection".to_string(),
            CertificationOrTraining::ManagementTraining => "ManagementTraining".to_string(),
            CertificationOrTraining::MentorshipProgram => "MentorshipProgram".to_string(),
            CertificationOrTraining::TechnicalCertification => "TechnicalCertification".to_string(),
            CertificationOrTraining::Other(x) => x,
        }
    }
}

impl From<String> for CertificationOrTraining {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "FirstAid" => CertificationOrTraining::FirstAid,
            "TeachingCertificate" => CertificationOrTraining::TeachingCertificate,
            "ComputerLiteracy" => CertificationOrTraining::ComputerLiteracy,
            "LeadershipTraining" => CertificationOrTraining::LeadershipTraining,
            "SafetyTraining" => CertificationOrTraining::SafetyTraining,
            "LanguageProficiency" => CertificationOrTraining::LanguageProficiency,
            "CounselingTraining" => CertificationOrTraining::CounselingTraining,
            "ChildProtection" => CertificationOrTraining::ChildProtection,
            "ManagementTraining" => CertificationOrTraining::ManagementTraining,
            "MentorshipProgram" => CertificationOrTraining::MentorshipProgram,
            "TechnicalCertification" => CertificationOrTraining::TechnicalCertification,
            _ => CertificationOrTraining::Other(raw),
        }
    }
}

impl fmt::Display for CertificationOrTraining {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertificationOrTraining::FirstAid => write!(f, "FirstAid"),
            CertificationOrTraining::TeachingCertificate => write!(f, "TeachingCertificate"),
            CertificationOrTraining::ComputerLiteracy => write!(f, "ComputerLiteracy"),
            CertificationOrTraining::LeadershipTraining => write!(f, "LeadershipTraining"),
            CertificationOrTraining::SafetyTraining => write!(f, "SafetyTraining"),
            CertificationOrTraining::LanguageProficiency => write!(f, "LanguageProficiency"),
            CertificationOrTraining::CounselingTraining => write!(f, "CounselingTraining"),
            CertificationOrTraining::ChildProtection => write!(f, "ChildProtection"),
            CertificationOrTraining::ManagementTraining => write!(f, "ManagementTraining"),
            CertificationOrTraining::MentorshipProgram => write!(f, "MentorshipProgram"),
            CertificationOrTraining::TechnicalCertification => write!(f, "TechnicalCertification"),
            CertificationOrTraining::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
pub enum TeachingStyle {
    Lecture,
    Discussion,
    HandsOn,
    ProjectBased,
    Flipped,
    Collaborative,
    Individualized,
    Digital,
    Other(String),
}

impl From<TeachingStyle> for String {
    fn from(t: TeachingStyle) -> String {
        match t {
            TeachingStyle::Lecture => "Lecture".to_string(),
            TeachingStyle::Discussion => "Discussion".to_string(),
            TeachingStyle::HandsOn => "HandsOn".to_string(),
            TeachingStyle::ProjectBased => "ProjectBased".to_string(),
            TeachingStyle::Flipped => "Flipped".to_string(),
            TeachingStyle::Collaborative => "Collaborative".to_string(),
            TeachingStyle::Individualized => "Individualized".to_string(),
            TeachingStyle::Digital => "Digital".to_string(),
            TeachingStyle::Other(x) => x,
        }
    }
}

impl From<String> for TeachingStyle {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Lecture" => TeachingStyle::Lecture,
            "Discussion" => TeachingStyle::Discussion,
            "HandsOn" => TeachingStyle::HandsOn,
            "ProjectBased" => TeachingStyle::ProjectBased,
            "Flipped" => TeachingStyle::Flipped,
            "Collaborative" => TeachingStyle::Collaborative,
            "Individualized" => TeachingStyle::Individualized,
            "Digital" => TeachingStyle::Digital,
            _ => TeachingStyle::Other(raw),
        }
    }
}

impl fmt::Display for TeachingStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TeachingStyle::Lecture => write!(f, "Lecture"),
            TeachingStyle::Discussion => write!(f, "Discussion"),
            TeachingStyle::HandsOn => write!(f, "HandsOn"),
            TeachingStyle::ProjectBased => write!(f, "ProjectBased"),
            TeachingStyle::Flipped => write!(f, "Flipped"),
            TeachingStyle::Collaborative => write!(f, "Collaborative"),
            TeachingStyle::Individualized => write!(f, "Individualized"),
            TeachingStyle::Digital => write!(f, "Digital"),
            TeachingStyle::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
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

impl From<AgeGroup> for String {
    fn from(a: AgeGroup) -> String {
        match a {
            AgeGroup::Age6To9 => "Age6To9".to_string(),
            AgeGroup::Age10To12 => "Age10To12".to_string(),
            AgeGroup::Age13To15 => "Age13To15".to_string(),
            AgeGroup::Age16To18 => "Age16To18".to_string(),
            AgeGroup::Grade1To3 => "Grade1To3".to_string(),
            AgeGroup::Grade4To6 => "Grade4To6".to_string(),
            AgeGroup::Grade7To9 => "Grade7To9".to_string(),
            AgeGroup::Grade10To12 => "Grade10To12".to_string(),
            AgeGroup::AdultEducation => "AdultEducation".to_string(),
            AgeGroup::Other(x) => x,
        }
    }
}

impl From<String> for AgeGroup {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Age6To9" => AgeGroup::Age6To9,
            "Age10To12" => AgeGroup::Age10To12,
            "Age13To15" => AgeGroup::Age13To15,
            "Age16To18" => AgeGroup::Age16To18,
            "Grade1To3" => AgeGroup::Grade1To3,
            "Grade4To6" => AgeGroup::Grade4To6,
            "Grade7To9" => AgeGroup::Grade7To9,
            "Grade10To12" => AgeGroup::Grade10To12,
            "AdultEducation" => AgeGroup::AdultEducation,
            _ => AgeGroup::Other(raw),
        }
    }
}

impl fmt::Display for AgeGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgeGroup::Age6To9 => write!(f, "Age6To9"),
            AgeGroup::Age10To12 => write!(f, "Age10To12"),
            AgeGroup::Age13To15 => write!(f, "Age13To15"),
            AgeGroup::Age16To18 => write!(f, "Age16To18"),
            AgeGroup::Grade1To3 => write!(f, "Grade1To3"),
            AgeGroup::Grade4To6 => write!(f, "Grade4To6"),
            AgeGroup::Grade7To9 => write!(f, "Grade7To9"),
            AgeGroup::Grade10To12 => write!(f, "Grade10To12"),
            AgeGroup::AdultEducation => write!(f, "AdultEducation"),
            AgeGroup::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
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

impl From<ProfessionalGoal> for String {
    fn from(p: ProfessionalGoal) -> String {
        match p {
            ProfessionalGoal::ImproveDigitalSkills => "ImproveDigitalSkills".to_string(),
            ProfessionalGoal::MentorStudents => "MentorStudents".to_string(),
            ProfessionalGoal::ClassroomManagement => "ClassroomManagement".to_string(),
            ProfessionalGoal::CurriculumDevelopment => "CurriculumDevelopment".to_string(),
            ProfessionalGoal::AssessmentSkills => "AssessmentSkills".to_string(),
            ProfessionalGoal::InclusiveEducation => "InclusiveEducation".to_string(),
            ProfessionalGoal::LeadershipTraining => "LeadershipTraining".to_string(),
            ProfessionalGoal::Other(x) => x,
        }
    }
}

impl From<String> for ProfessionalGoal {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "ImproveDigitalSkills" => ProfessionalGoal::ImproveDigitalSkills,
            "MentorStudents" => ProfessionalGoal::MentorStudents,
            "ClassroomManagement" => ProfessionalGoal::ClassroomManagement,
            "CurriculumDevelopment" => ProfessionalGoal::CurriculumDevelopment,
            "AssessmentSkills" => ProfessionalGoal::AssessmentSkills,
            "InclusiveEducation" => ProfessionalGoal::InclusiveEducation,
            "LeadershipTraining" => ProfessionalGoal::LeadershipTraining,
            _ => ProfessionalGoal::Other(raw),
        }
    }
}

impl fmt::Display for ProfessionalGoal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfessionalGoal::ImproveDigitalSkills => write!(f, "ImproveDigitalSkills"),
            ProfessionalGoal::MentorStudents => write!(f, "MentorStudents"),
            ProfessionalGoal::ClassroomManagement => write!(f, "ClassroomManagement"),
            ProfessionalGoal::CurriculumDevelopment => write!(f, "CurriculumDevelopment"),
            ProfessionalGoal::AssessmentSkills => write!(f, "AssessmentSkills"),
            ProfessionalGoal::InclusiveEducation => write!(f, "InclusiveEducation"),
            ProfessionalGoal::LeadershipTraining => write!(f, "LeadershipTraining"),
            ProfessionalGoal::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
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

impl From<Department> for String {
    fn from(d: Department) -> String {
        match d {
            Department::Administration => "Administration".to_string(),
            Department::Finance => "Finance".to_string(),
            Department::Library => "Library".to_string(),
            Department::IT => "IT".to_string(),
            Department::HR => "HR".to_string(),
            Department::Maintenance => "Maintenance".to_string(),
            Department::Security => "Security".to_string(),
            Department::Cafeteria => "Cafeteria".to_string(),
            Department::Transport => "Transport".to_string(),
            Department::Other(x) => x,
        }
    }
}

impl From<String> for Department {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Administration" => Department::Administration,
            "Finance" => Department::Finance,
            "Library" => Department::Library,
            "IT" => Department::IT,
            "HR" => Department::HR,
            "Maintenance" => Department::Maintenance,
            "Security" => Department::Security,
            "Cafeteria" => Department::Cafeteria,
            "Transport" => Department::Transport,
            _ => Department::Other(raw),
        }
    }
}

impl fmt::Display for Department {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Department::Administration => write!(f, "Administration"),
            Department::Finance => write!(f, "Finance"),
            Department::Library => write!(f, "Library"),
            Department::IT => write!(f, "IT"),
            Department::HR => write!(f, "HR"),
            Department::Maintenance => write!(f, "Maintenance"),
            Department::Security => write!(f, "Security"),
            Department::Cafeteria => write!(f, "Cafeteria"),
            Department::Transport => write!(f, "Transport"),
            Department::Other(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(from = "String", into = "String")]
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

impl From<JobTitle> for String {
    fn from(j: JobTitle) -> String {
        match j {
            JobTitle::Accountant => "Accountant".to_string(),
            JobTitle::Secretary => "Secretary".to_string(),
            JobTitle::Clerk => "Clerk".to_string(),
            JobTitle::Librarian => "Librarian".to_string(),
            JobTitle::SecurityGuard => "SecurityGuard".to_string(),
            JobTitle::ITSupport => "ITSupport".to_string(),
            JobTitle::Manager => "Manager".to_string(),
            JobTitle::Teacher => "Teacher".to_string(),
            JobTitle::Counselor => "Counselor".to_string(),
            JobTitle::Other(x) => x,
        }
    }
}

impl From<String> for JobTitle {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.as_str() {
            "Accountant" => JobTitle::Accountant,
            "Secretary" => JobTitle::Secretary,
            "Clerk" => JobTitle::Clerk,
            "Librarian" => JobTitle::Librarian,
            "SecurityGuard" => JobTitle::SecurityGuard,
            "ITSupport" => JobTitle::ITSupport,
            "Manager" => JobTitle::Manager,
            "Teacher" => JobTitle::Teacher,
            "Counselor" => JobTitle::Counselor,
            _ => JobTitle::Other(raw),
        }
    }
}

impl fmt::Display for JobTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobTitle::Accountant => write!(f, "Accountant"),
            JobTitle::Secretary => write!(f, "Secretary"),
            JobTitle::Clerk => write!(f, "Clerk"),
            JobTitle::Librarian => write!(f, "Librarian"),
            JobTitle::SecurityGuard => write!(f, "SecurityGuard"),
            JobTitle::ITSupport => write!(f, "ITSupport"),
            JobTitle::Manager => write!(f, "Manager"),
            JobTitle::Teacher => write!(f, "Teacher"),
            JobTitle::Counselor => write!(f, "Counselor"),
            JobTitle::Other(value) => write!(f, "{}", value),
        }
    }
}

// ------------------ other structs ------------------ //

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl TimeRange {
    pub fn new(start: &str, end: &str) -> Self {
        Self {
            start: NaiveTime::parse_from_str(start, "%H:%M")
                .expect("Invalid start time format (expected HH:MM)"),
            end: NaiveTime::parse_from_str(end, "%H:%M")
                .expect("Invalid end time format (expected HH:MM)"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyAvailability {
    pub day: Weekday,
    pub time_range: TimeRange,
}

// 🔹 Image struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub id: String,
    pub url: String,
}
