xThis document defines the **step-by-step backend implementation order** for **Space-Together**.

Each version builds upon the previous one.

---

## ✅ Completed (v0.0.3)

- User CRUD
- JWT Authentication
- Role-Based Access Control (RBAC)
- Error handling

---

## v0.0.4 — Education Base Models

### 🏫 Step 1: Sector

- [x] Create `Sector` collection
- [x] Implement CRUD APIs (only Director/Admin can manage)
- [x] Seed default values (REB, TVET, etc.)

```rust
pub struct Sector {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub curriculum: Option<(i32, i32)>, // start-end years
    pub country: String,
    pub r#type: String, // global, international, local
}

```

---

### 🧰 Step 2: Trade

- [x] Create `Trade` collection (linked to Sector)
- [x] CRUD APIs (linked to `Sector`)

```rust
pub struct Trade {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub sector_id: ObjectId,
    pub trade_id: Option<ObjectId>, // self relation
    pub class_range: (i32, i32),
    pub r#type: String, // Senior, Primary, Nursing, etc.
}

```

---

### 🏫 Step 3: Main Class

- [x] Create `MainClass` collection (linked to Trade)
- [x] CRUD APIs

```rust
pub struct MainClass {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub username: String,
    pub trade_id: ObjectId,
    pub description: Option<String>,
}

```

---

## 🧩 Step 4: Main Subject (+ Resources)

The **Main Subject** is the central structure that connects all academic resources — including learning outcomes, topics, materials, grading schemes, and progress tracking.

---

### 🧠 Main Subject

Represents a complete subject offered within a class or trade. It defines its structure, prerequisites, contributors, and academic details.

```rust
pub struct MainSubject {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub level: Option<String>,
    pub estimated_hours: i32,
    pub credits: Option<i32>,
    pub category: SubjectCategory, // e.g. Science, TVET, Humanities
    pub main_class_ids: Vec<ObjectId>,
    pub prerequisites: Option<Vec<ObjectId>>,
    pub contributors: Vec<SubjectContributor>,
    pub starting_year: Option<DateTime<Utc>>,
    pub ending_year: Option<DateTime<Utc>>,
    pub created_by: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

```

---

### 📂 Related Entities

Each `MainSubject` can include several **sub-resources** that define its academic framework.

---

Defines what students should achieve after completing a subject or a section.

```rust
pub struct LearningOutcome {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub subject_id: ObjectId,
    pub title: String,
    pub description: Option<String>,
    pub order: i32,
    pub estimated_hours: Option<i32>,
    pub key_competencies: SubjectCompetencyBlock,
    pub assessment_criteria: Vec<String>,
    pub role: SubjectTypeFor, // MainSubject | ClassSubject
    pub prerequisites: Option<Vec<ObjectId>>,
    pub is_mandatory: Option<bool>,
    pub created_by: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

```

---

Topics break down learning outcomes into specific, teachable components.

They can be **nested** (topics → subtopics).

```rust
pub struct SubjectTopic {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub learning_outcome_id: Option<ObjectId>,
    pub parent_topic_id: Option<ObjectId>,
    pub title: String,
    pub description: Option<String>,
    pub order: f32,
    pub created_by: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

```

Nested (for population):

```rust
pub struct SubjectTopicWithOthers {
    pub topic: SubjectTopic,
    pub sub_topics: Option<Vec<SubjectTopicWithOthers>>,
    pub learning_materials: Option<Vec<SubjectLearningMaterial>>,
}

```

---

Attach books, videos, PDFs, or digital content to any subject, topic, or learning outcome.

```rust
pub struct SubjectLearningMaterial {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub material_type: SubjectMaterialType, // Book, Video, etc.
    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub role: SubjectLearningMaterialRole, // MainSubject, LearningOutcome, etc.
    pub reference_id: Option<ObjectId>,
    pub created_by: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

```

---

Defines grading boundaries, weighting, and the minimum passing grade.

```rust
pub struct SubjectGradingScheme {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub reference_id: Option<ObjectId>,
    pub scheme_type: SubjectGradingType, // LetterGrade, Percentage, etc.
    pub grade_boundaries: HashMap<String, f32>,
    pub assessment_weights: HashMap<String, f32>,
    pub minimum_passing_grade: String,
    pub role: SubjectTypeFor, // MainSubject | ClassSubject
    pub created_by: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

```

---

Controls attendance, assignment tracking, and performance thresholds.

```rust
pub struct SubjectProgressTrackingConfig {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub reference_id: Option<ObjectId>,
    pub track_attendance: bool,
    pub track_assignments: bool,
    pub track_topic_coverage: bool,
    pub track_skill_acquisition: bool,
    pub thresholds: SubjectProgressThresholds,
    pub role: SubjectProgressTrackingConfigType, // MainSubject | ClassSubject
    pub created_by: Option<ObjectId>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

```

---

Defines the subject authors, reviewers, or collaborators.

```rust
pub struct SubjectContributor {
    pub name: String,
    pub role: String, // Author, Reviewer, etc.
    pub user_id: Option<ObjectId>,
}

```

---

Defines the expected knowledge, skills, and attitudes from the learning outcome.

```rust
pub struct SubjectCompetencyBlock {
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
    pub attitudes: Vec<String>,
}

```

---

### 🧱 Combined View

When fetching a subject with its relationships:

```rust
pub struct MainSubjectWithOthers {
    pub subject: MainSubject,
    pub learning_outcome: Option<Vec<LearningOutcomeWithOthers>>,
    pub progress_tracking_config: Option<SubjectProgressTrackingConfig>,
    pub grading_schemes: Option<SubjectGradingScheme>,
}

```

---

## v0.0.5 — School Management

### 🏫 Step 5: School

- [ ] Create `School` collection
- [ ] CRUD APIs (only Director/Head of Studies)

```rust
pub struct School {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub username: String,
    pub location: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub contact: Option<String>,
    pub owner_id: ObjectId,
    pub sectors: Vec<ObjectId>,
    pub trades: Vec<ObjectId>,
    pub main_classes: Vec<ObjectId>,
}

```

---

### 👥 Step 6: School Member

⚠️ Depends on **School + User**.

- [ ] Create `SchoolMember` collection
- [ ] Assign **roles**: Director, Head of Studies, Teacher, Student
- [ ] CRUD APIs (only Director can manage staff)

```rust
pub struct SchoolMember {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub school_id: ObjectId,
    pub name: String,
    pub image: Option<String>,
    pub r#type: String, // Director, HeadOfStudies, Teacher, Student
}

```

---

## v0.0.6 — Classes & Subjects

### 🏫 Step 7: Class

- [ ] Auto-create classes from **School + MainClass**
- [ ] CRUD APIs
- [ ] Assign **class teacher**

```rust
pub struct Class {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub username: String,
    pub school_id: Option<ObjectId>,
    pub class_teacher_id: Option<ObjectId>,
    pub r#type: String, // private, school, public
    pub main_class_id: ObjectId,
}

```

---

### 📘 Step 8: Subject

- [ ] Auto-create subjects from **MainSubject + Class**
- [ ] CRUD APIs
- [ ] Assign subject teachers

```rust
pub struct Subject {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub username: String,
    pub class_id: ObjectId,
    pub class_teacher_id: Option<ObjectId>,
    pub main_subject_id: ObjectId,
}

```

---

## v0.0.7 — Learning

### 🗒️ Step 9: Notes

- [ ] Create `Note` collection
- [ ] Teachers can post notes → students access

```rust
pub struct Note {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub teacher_id: ObjectId,
    pub subject_id: ObjectId,
    pub topic: String,
    pub content_id: ObjectId,
    pub title: String,
    pub description: Option<String>,
    pub class_id: ObjectId,
}

```

---

### 🧮 Step 10: Assessment

- [ ] Create `Assessment` collection
- [ ] Linked to class + teacher + content

```rust
pub struct Assessment {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub teacher_id: ObjectId,
    pub class_id: ObjectId,
    pub content_id: ObjectId,
    pub title: String,
    pub marks: i32,
}

```

---

### 📄 Step 11: Content

- [ ] Create `Content` collection (reusable blocks)

```rust
pub struct Content {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub r#type: String, // text, tutorial, link, question
    pub content: String,
}

```

---

## v0.0.8 — Social & Groups

### 📰 Step 12: Post

- [ ] Create `Post` collection (announcements, events)

```rust
pub struct Post {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub title: String,
    pub description: String,
    pub content_id: Option<ObjectId>,
    pub school_id: Option<ObjectId>,
    pub class_id: Option<ObjectId>,
}

```

---

### 👨‍👩‍👧 Step 13: Class Group

- [ ] Create `ClassGroup` collection (study/work groups)

```rust
pub struct ClassGroup {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: Option<String>,
    pub owner_type: String, // teacher, staff, student
    pub subject_id: Option<ObjectId>,
    pub members: Vec<ObjectId>,
    pub class_id: ObjectId,
    pub date: (DateTime, DateTime),
    pub r#type: String, // permanent, temporary
    pub assessment_id: Option<ObjectId>,
    pub leader_id: Option<ObjectId>,
}

```

---

## v0.0.9 — Teachers & Students

### 👩🏽‍🏫 Step 14: Teacher

- [ ] Link teachers to schools

```rust
pub struct Teacher {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub image: Option<String>,
    pub school_id: ObjectId,
    pub user_id: ObjectId,
}

```

---

### 👨🏽‍🎓 Step 15: Student

- [ ] Link students to schools

```rust
pub struct Student {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub image: Option<String>,
    pub school_id: ObjectId,
    pub user_id: ObjectId,
}

```

---

🔥 **Build Order Summary**

1. Education Base (Sector → Trade → MainClass → MainSubject)
2. School → Members
3. Class → Subject
4. Learning (Notes, Assessment, Content)
5. Social (Posts, Groups)
6. People (Teacher, Student)

---
