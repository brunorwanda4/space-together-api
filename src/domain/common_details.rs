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
            StudyStyle::Visual => "VISUAL".to_string(),
            StudyStyle::Discussion => "DISCUSSION".to_string(),
            StudyStyle::HandsOn => "HANDSON".to_string(),
            StudyStyle::Reading => "READING".to_string(),
            StudyStyle::Writing => "WRITING".to_string(),
            StudyStyle::Group => "GROUP".to_string(),
            StudyStyle::Solo => "SOLO".to_string(),
            StudyStyle::ProjectBased => "PROJECTBASED".to_string(),
            StudyStyle::Digital => "DIGITAL".to_string(),
            StudyStyle::Other(x) => x,
        }
    }
}

impl From<String> for StudyStyle {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "VISUAL" => StudyStyle::Visual,
            "DISCUSSION" => StudyStyle::Discussion,
            "HANDSON" => StudyStyle::HandsOn,
            "READING" => StudyStyle::Reading,
            "WRITING" => StudyStyle::Writing,
            "GROUP" => StudyStyle::Group,
            "SOLO" => StudyStyle::Solo,
            "PROJECTBASED" => StudyStyle::ProjectBased,
            "DIGITAL" => StudyStyle::Digital,
            _ => StudyStyle::Other(raw),
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
            CommunicationMethod::Chat => "CHAT".to_string(),
            CommunicationMethod::Sms => "SMS".to_string(),
            CommunicationMethod::Email => "EMAIL".to_string(),
            CommunicationMethod::Call => "CALL".to_string(),
            CommunicationMethod::VideoCall => "VIDEOCALL".to_string(),
            CommunicationMethod::InPerson => "INPERSON".to_string(),
            CommunicationMethod::Other(x) => x,
        }
    }
}

impl From<String> for CommunicationMethod {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "CHAT" => CommunicationMethod::Chat,
            "SMS" => CommunicationMethod::Sms,
            "EMAIL" => CommunicationMethod::Email,
            "CALL" => CommunicationMethod::Call,
            "VIDEOCALL" => CommunicationMethod::VideoCall,
            "INPERSON" => CommunicationMethod::InPerson,
            _ => CommunicationMethod::Other(raw),
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
            Relationship::Parent => "PARENT".to_string(),
            Relationship::Mother => "MOTHER".to_string(),
            Relationship::Father => "FATHER".to_string(),
            Relationship::StepMother => "STEPMOTHER".to_string(),
            Relationship::StepFather => "STEPFATHER".to_string(),
            Relationship::Grandmother => "GRANDMOTHER".to_string(),
            Relationship::Grandfather => "GRANDFATHER".to_string(),
            Relationship::Aunt => "AUNT".to_string(),
            Relationship::Uncle => "UNCLE".to_string(),
            Relationship::Brother => "BROTHER".to_string(),
            Relationship::Sister => "SISTER".to_string(),
            Relationship::Cousin => "COUSIN".to_string(),
            Relationship::Guardian => "GUARDIAN".to_string(),
            Relationship::Sponsor => "SPONSOR".to_string(),
            Relationship::Caregiver => "CAREGIVER".to_string(),
            Relationship::FosterParent => "FOSTERPARENT".to_string(),
            Relationship::HostParent => "HOSTPARENT".to_string(),
            Relationship::Mentor => "MENTOR".to_string(),
            Relationship::Teacher => "TEACHER".to_string(),
            Relationship::Neighbor => "NEIGHBOR".to_string(),
            Relationship::FamilyFriend => "FAMILYFRIEND".to_string(),
            Relationship::LegalRepresentative => "LEGALREPRESENTATIVE".to_string(),
            Relationship::SocialWorker => "SOCIALWORKER".to_string(),
            Relationship::Other(x) => x,
        }
    }
}

impl From<String> for Relationship {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "PARENT" => Relationship::Parent,
            "MOTHER" => Relationship::Mother,
            "FATHER" => Relationship::Father,
            "STEPMOTHER" => Relationship::StepMother,
            "STEPFATHER" => Relationship::StepFather,
            "GRANDMOTHER" => Relationship::Grandmother,
            "GRANDFATHER" => Relationship::Grandfather,
            "AUNT" => Relationship::Aunt,
            "UNCLE" => Relationship::Uncle,
            "BROTHER" => Relationship::Brother,
            "SISTER" => Relationship::Sister,
            "COUSIN" => Relationship::Cousin,
            "GUARDIAN" => Relationship::Guardian,
            "SPONSOR" => Relationship::Sponsor,
            "CAREGIVER" => Relationship::Caregiver,
            "FOSTERPARENT" => Relationship::FosterParent,
            "HOSTPARENT" => Relationship::HostParent,
            "MENTOR" => Relationship::Mentor,
            "TEACHER" => Relationship::Teacher,
            "NEIGHBOR" => Relationship::Neighbor,
            "FAMILYFRIEND" => Relationship::FamilyFriend,
            "LEGALREPRESENTATIVE" => Relationship::LegalRepresentative,
            "SOCIALWORKER" => Relationship::SocialWorker,
            _ => Relationship::Other(raw),
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
            SpecialSupport::Financial => "FINANCIAL".to_string(),
            SpecialSupport::Academic => "ACADEMIC".to_string(),
            SpecialSupport::Emotional => "EMOTIONAL".to_string(),
            SpecialSupport::Medical => "MEDICAL".to_string(),
            SpecialSupport::Mobility => "MOBILITY".to_string(),
            SpecialSupport::Nutritional => "NUTRITIONAL".to_string(),
            SpecialSupport::Social => "SOCIAL".to_string(),
            SpecialSupport::Language => "LANGUAGE".to_string(),
            SpecialSupport::Technical => "TECHNICAL".to_string(),
            SpecialSupport::Other(x) => x,
        }
    }
}

impl From<String> for SpecialSupport {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "FINANCIAL" => SpecialSupport::Financial,
            "ACADEMIC" => SpecialSupport::Academic,
            "EMOTIONAL" => SpecialSupport::Emotional,
            "MEDICAL" => SpecialSupport::Medical,
            "MOBILITY" => SpecialSupport::Mobility,
            "NUTRITIONAL" => SpecialSupport::Nutritional,
            "SOCIAL" => SpecialSupport::Social,
            "LANGUAGE" => SpecialSupport::Language,
            "TECHNICAL" => SpecialSupport::Technical,
            _ => SpecialSupport::Other(raw),
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
            LearningChallenge::NeedsTutoring => "NEEDSTUTORING".to_string(),
            LearningChallenge::LanguageSupport => "LANGUAGESUPPORT".to_string(),
            LearningChallenge::LiteracyDifficulty => "LITERACYDIFFICULTY".to_string(),
            LearningChallenge::AttentionDifficulty => "ATTENTIONDIFFICULTY".to_string(),
            LearningChallenge::HearingImpairment => "HEARINGIMPAIRMENT".to_string(),
            LearningChallenge::VisualImpairment => "VISUALIMPAIRMENT".to_string(),
            LearningChallenge::PhysicalDisability => "PHYSICALDISABILITY".to_string(),
            LearningChallenge::BehavioralDifficulty => "BEHAVIORALDIFFICULTY".to_string(),
            LearningChallenge::MathDifficulty => "MATHDIFFICULTY".to_string(),
            LearningChallenge::LearningDisability => "LEARNINGDISABILITY".to_string(),
            LearningChallenge::StudySkillsSupport => "STUDYSKILLSSUPPORT".to_string(),
            LearningChallenge::Other(x) => x,
        }
    }
}

impl From<String> for LearningChallenge {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "NEEDSTUTORING" => LearningChallenge::NeedsTutoring,
            "LANGUAGESUPPORT" => LearningChallenge::LanguageSupport,
            "LITERACYDIFFICULTY" => LearningChallenge::LiteracyDifficulty,
            "ATTENTIONDIFFICULTY" => LearningChallenge::AttentionDifficulty,
            "HEARINGIMPAIRMENT" => LearningChallenge::HearingImpairment,
            "VISUALIMPAIRMENT" => LearningChallenge::VisualImpairment,
            "PHYSICALDISABILITY" => LearningChallenge::PhysicalDisability,
            "BEHAVIORALDIFFICULTY" => LearningChallenge::BehavioralDifficulty,
            "MATHDIFFICULTY" => LearningChallenge::MathDifficulty,
            "LEARNINGDISABILITY" => LearningChallenge::LearningDisability,
            "STUDYSKILLSSUPPORT" => LearningChallenge::StudySkillsSupport,
            _ => LearningChallenge::Other(raw),
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
            EmploymentType::FullTime => "FULLTIME".to_string(),
            EmploymentType::PartTime => "PARTTIME".to_string(),
            EmploymentType::Volunteer => "VOLUNTEER".to_string(),
            EmploymentType::Contract => "CONTRACT".to_string(),
            EmploymentType::Internship => "INTERNSHIP".to_string(),
            EmploymentType::SelfEmployed => "SELFEMPLOYED".to_string(),
            EmploymentType::Unemployed => "UNEMPLOYED".to_string(),
            EmploymentType::Other(x) => x,
        }
    }
}

impl From<String> for EmploymentType {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "FULLTIME" => EmploymentType::FullTime,
            "PARTTIME" => EmploymentType::PartTime,
            "VOLUNTEER" => EmploymentType::Volunteer,
            "CONTRACT" => EmploymentType::Contract,
            "INTERNSHIP" => EmploymentType::Internship,
            "SELFEMPLOYED" => EmploymentType::SelfEmployed,
            "UNEMPLOYED" => EmploymentType::Unemployed,
            _ => EmploymentType::Other(raw),
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
            EducationLevel::None => "NONE".to_string(),
            EducationLevel::Primary => "PRIMARY".to_string(),
            EducationLevel::HighSchool => "HIGHSCHOOL".to_string(),
            EducationLevel::Vocational => "VOCATIONAL".to_string(),
            EducationLevel::Diploma => "DIPLOMA".to_string(),
            EducationLevel::Bachelor => "BACHELOR".to_string(),
            EducationLevel::Master => "MASTER".to_string(),
            EducationLevel::Doctorate => "DOCTORATE".to_string(),
            EducationLevel::Professional => "PROFESSIONAL".to_string(),
            EducationLevel::InProgress => "INPROGRESS".to_string(),
            EducationLevel::Other(x) => x,
        }
    }
}

impl From<String> for EducationLevel {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "NONE" => EducationLevel::None,
            "PRIMARY" => EducationLevel::Primary,
            "HIGHSCHOOL" => EducationLevel::HighSchool,
            "VOCATIONAL" => EducationLevel::Vocational,
            "DIPLOMA" => EducationLevel::Diploma,
            "BACHELOR" => EducationLevel::Bachelor,
            "MASTER" => EducationLevel::Master,
            "DOCTORATE" => EducationLevel::Doctorate,
            "PROFESSIONAL" => EducationLevel::Professional,
            "INPROGRESS" => EducationLevel::InProgress,
            _ => EducationLevel::Other(raw),
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
            CertificationOrTraining::FirstAid => "FIRSTAID".to_string(),
            CertificationOrTraining::TeachingCertificate => "TEACHINGCERTIFICATE".to_string(),
            CertificationOrTraining::ComputerLiteracy => "COMPUTERLITERACY".to_string(),
            CertificationOrTraining::LeadershipTraining => "LEADERSHIPTRAINING".to_string(),
            CertificationOrTraining::SafetyTraining => "SAFETYTRAINING".to_string(),
            CertificationOrTraining::LanguageProficiency => "LANGUAGEPROFICIENCY".to_string(),
            CertificationOrTraining::CounselingTraining => "COUNSELINGTRAINING".to_string(),
            CertificationOrTraining::ChildProtection => "CHILDPROTECTION".to_string(),
            CertificationOrTraining::ManagementTraining => "MANAGEMENTTRAINING".to_string(),
            CertificationOrTraining::MentorshipProgram => "MENTORSHIPPROGRAM".to_string(),
            CertificationOrTraining::TechnicalCertification => "TECHNICALCERTIFICATION".to_string(),
            CertificationOrTraining::Other(x) => x,
        }
    }
}

impl From<String> for CertificationOrTraining {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "FIRSTAID" => CertificationOrTraining::FirstAid,
            "TEACHINGCERTIFICATE" => CertificationOrTraining::TeachingCertificate,
            "COMPUTERLITERACY" => CertificationOrTraining::ComputerLiteracy,
            "LEADERSHIPTRAINING" => CertificationOrTraining::LeadershipTraining,
            "SAFETYTRAINING" => CertificationOrTraining::SafetyTraining,
            "LANGUAGEPROFICIENCY" => CertificationOrTraining::LanguageProficiency,
            "COUNSELINGTRAINING" => CertificationOrTraining::CounselingTraining,
            "CHILDPROTECTION" => CertificationOrTraining::ChildProtection,
            "MANAGEMENTTRAINING" => CertificationOrTraining::ManagementTraining,
            "MENTORSHIPPROGRAM" => CertificationOrTraining::MentorshipProgram,
            "TECHNICALCERTIFICATION" => CertificationOrTraining::TechnicalCertification,
            _ => CertificationOrTraining::Other(raw),
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
            TeachingStyle::Lecture => "LECTURE".to_string(),
            TeachingStyle::Discussion => "DISCUSSION".to_string(),
            TeachingStyle::HandsOn => "HANDSON".to_string(),
            TeachingStyle::ProjectBased => "PROJECTBASED".to_string(),
            TeachingStyle::Flipped => "FLIPPED".to_string(),
            TeachingStyle::Collaborative => "COLLABORATIVE".to_string(),
            TeachingStyle::Individualized => "INDIVIDUALIZED".to_string(),
            TeachingStyle::Digital => "DIGITAL".to_string(),
            TeachingStyle::Other(x) => x,
        }
    }
}

impl From<String> for TeachingStyle {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "LECTURE" => TeachingStyle::Lecture,
            "DISCUSSION" => TeachingStyle::Discussion,
            "HANDSON" => TeachingStyle::HandsOn,
            "PROJECTBASED" => TeachingStyle::ProjectBased,
            "FLIPPED" => TeachingStyle::Flipped,
            "COLLABORATIVE" => TeachingStyle::Collaborative,
            "INDIVIDUALIZED" => TeachingStyle::Individualized,
            "DIGITAL" => TeachingStyle::Digital,
            _ => TeachingStyle::Other(raw),
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
            AgeGroup::Age6To9 => "AGE6TO9".to_string(),
            AgeGroup::Age10To12 => "AGE10TO12".to_string(),
            AgeGroup::Age13To15 => "AGE13TO15".to_string(),
            AgeGroup::Age16To18 => "AGE16TO18".to_string(),
            AgeGroup::Grade1To3 => "GRADE1TO3".to_string(),
            AgeGroup::Grade4To6 => "GRADE4TO6".to_string(),
            AgeGroup::Grade7To9 => "GRADE7TO9".to_string(),
            AgeGroup::Grade10To12 => "GRADE10TO12".to_string(),
            AgeGroup::AdultEducation => "ADULTEDUCATION".to_string(),
            AgeGroup::Other(x) => x,
        }
    }
}

impl From<String> for AgeGroup {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "AGE6TO9" => AgeGroup::Age6To9,
            "AGE10TO12" => AgeGroup::Age10To12,
            "AGE13TO15" => AgeGroup::Age13To15,
            "AGE16TO18" => AgeGroup::Age16To18,
            "GRADE1TO3" => AgeGroup::Grade1To3,
            "GRADE4TO6" => AgeGroup::Grade4To6,
            "GRADE7TO9" => AgeGroup::Grade7To9,
            "GRADE10TO12" => AgeGroup::Grade10To12,
            "ADULTEDUCATION" => AgeGroup::AdultEducation,
            _ => AgeGroup::Other(raw),
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
            ProfessionalGoal::ImproveDigitalSkills => "IMPROVEDIGITALSKILLS".to_string(),
            ProfessionalGoal::MentorStudents => "MENTORSTUDENTS".to_string(),
            ProfessionalGoal::ClassroomManagement => "CLASSROOMMANAGEMENT".to_string(),
            ProfessionalGoal::CurriculumDevelopment => "CURRICULUMDEVELOPMENT".to_string(),
            ProfessionalGoal::AssessmentSkills => "ASSESSMENTSKILLS".to_string(),
            ProfessionalGoal::InclusiveEducation => "INCLUSIVEEDUCATION".to_string(),
            ProfessionalGoal::LeadershipTraining => "LEADERSHIPTRAINING".to_string(),
            ProfessionalGoal::Other(x) => x,
        }
    }
}

impl From<String> for ProfessionalGoal {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "IMPROVEDIGITALSKILLS" => ProfessionalGoal::ImproveDigitalSkills,
            "MENTORSTUDENTS" => ProfessionalGoal::MentorStudents,
            "CLASSROOMMANAGEMENT" => ProfessionalGoal::ClassroomManagement,
            "CURRICULUMDEVELOPMENT" => ProfessionalGoal::CurriculumDevelopment,
            "ASSESSMENTSKILLS" => ProfessionalGoal::AssessmentSkills,
            "INCLUSIVEEDUCATION" => ProfessionalGoal::InclusiveEducation,
            "LEADERSHIPTRAINING" => ProfessionalGoal::LeadershipTraining,
            _ => ProfessionalGoal::Other(raw),
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
            Department::Administration => "ADMINISTRATION".to_string(),
            Department::Finance => "FINANCE".to_string(),
            Department::Library => "LIBRARY".to_string(),
            Department::IT => "IT".to_string(),
            Department::HR => "HR".to_string(),
            Department::Maintenance => "MAINTENANCE".to_string(),
            Department::Security => "SECURITY".to_string(),
            Department::Cafeteria => "CAFETERIA".to_string(),
            Department::Transport => "TRANSPORT".to_string(),
            Department::Other(x) => x,
        }
    }
}

impl From<String> for Department {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "ADMINISTRATION" => Department::Administration,
            "FINANCE" => Department::Finance,
            "LIBRARY" => Department::Library,
            "IT" => Department::IT,
            "HR" => Department::HR,
            "MAINTENANCE" => Department::Maintenance,
            "SECURITY" => Department::Security,
            "CAFETERIA" => Department::Cafeteria,
            "TRANSPORT" => Department::Transport,
            _ => Department::Other(raw),
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
            JobTitle::Accountant => "ACCOUNTANT".to_string(),
            JobTitle::Secretary => "SECRETARY".to_string(),
            JobTitle::Clerk => "CLERK".to_string(),
            JobTitle::Librarian => "LIBRARIAN".to_string(),
            JobTitle::SecurityGuard => "SECURITYGUARD".to_string(),
            JobTitle::ITSupport => "ITSUPPORT".to_string(),
            JobTitle::Manager => "MANAGER".to_string(),
            JobTitle::Teacher => "TEACHER".to_string(),
            JobTitle::Counselor => "COUNSELOR".to_string(),
            JobTitle::Other(x) => x,
        }
    }
}

impl From<String> for JobTitle {
    fn from(s: String) -> Self {
        let raw = s.clone();
        match s.to_uppercase().as_str() {
            "ACCOUNTANT" => JobTitle::Accountant,
            "SECRETARY" => JobTitle::Secretary,
            "CLERK" => JobTitle::Clerk,
            "LIBRARIAN" => JobTitle::Librarian,
            "SECURITYGUARD" => JobTitle::SecurityGuard,
            "ITSUPPORT" => JobTitle::ITSupport,
            "MANAGER" => JobTitle::Manager,
            "TEACHER" => JobTitle::Teacher,
            "COUNSELOR" => JobTitle::Counselor,
            _ => JobTitle::Other(raw),
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
