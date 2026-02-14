# Academic Records & Assessment System - Design Document

## Overview
Complete academic records and assessment system for Space-Together following existing architecture patterns.

## Architecture Alignment
- **Framework**: Actix-web with async handlers
- **Database**: MongoDB with database-per-school multi-tenancy
- **Auth**: JWT with role-based guards (Admin, Staff, Teacher, Student)
- **Patterns**: Service layer, aggregation pipelines, soft deletes, pagination, audit logging
- **Integration**: Users, Classes, ClassSubjects, EducationYear, Students, Teachers

## Module Structure

### 1. Exam Management (`exams`)
**Domain**: `src/domain/exam.rs`
```rust
struct Exam {
    id: ObjectId,
    school_id: ObjectId,
    education_year_id: ObjectId,
    term_id: Option<String>, // Reference to Term in EducationYear
    class_id: Option<ObjectId>, // Optional: exam for specific class
    name: String, // "Term 1 Exam", "Midterm", "Final"
    description: Option<String>,
    exam_type: ExamType, // Continuous, Midterm, Final, Quiz
    status: ExamStatus, // Draft, Published, InProgress, Completed, Archived
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    created_by: ObjectId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_deleted: bool,
}

enum ExamType { Continuous, Midterm, Final, Quiz, Assignment }
enum ExamStatus { Draft, Published, InProgress, Completed, Archived }
```

**Endpoints**:
- `POST /api/exams` - Create exam (Admin, Staff)
- `GET /api/exams` - List exams with filters (pagination, education_year, class, status)
- `GET /api/exams/:id` - Get exam details
- `PUT /api/exams/:id` - Update exam (Admin, Staff)
- `DELETE /api/exams/:id` - Soft delete (Admin)
- `POST /api/exams/:id/publish` - Publish exam (Admin, Staff)

### 2. Assessment Categories (`assessment_categories`)
**Domain**: `src/domain/assessment_category.rs`
```rust
struct AssessmentCategory {
    id: ObjectId,
    school_id: ObjectId,
    class_subject_id: ObjectId,
    education_year_id: ObjectId,
    name: String, // "CAT", "Final", "Quiz", "Assignment", "Practical"
    code: String, // "CAT1", "FINAL", "QUIZ1"
    weight_percentage: f64, // 0-100
    description: Option<String>,
    created_by: ObjectId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_deleted: bool,
}
```

**Validation**: Total weight per class_subject_id + education_year_id must not exceed 100%

**Endpoints**:
- `POST /api/assessment-categories` - Create category (Admin, Staff)
- `GET /api/assessment-categories` - List by class_subject (pagination)
- `PUT /api/assessment-categories/:id` - Update (Admin, Staff)
- `DELETE /api/assessment-categories/:id` - Soft delete (Admin)
- `GET /api/assessment-categories/validate/:class_subject_id` - Validate total weight

### 3. Grading System (`grading_scales`)
**Domain**: `src/domain/grading_scale.rs`
```rust
struct GradingScale {
    id: ObjectId,
    school_id: ObjectId,
    education_year_id: ObjectId,
    name: String, // "Letter Grade A-F", "Percentage", "Competency"
    grading_type: GradingType,
    grade_boundaries: Vec<GradeBoundary>,
    is_active: bool,
    created_by: ObjectId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_deleted: bool,
}

enum GradingType { Letter, Percentage, Competency }

struct GradeBoundary {
    grade: String, // "A", "B", "Excellent", "90-100"
    min_score: f64,
    max_score: f64,
    gpa_value: Option<f64>, // For GPA calculation
    description: Option<String>,
}
```

**Endpoints**:
- `POST /api/grading-scales` - Create scale (Admin)
- `GET /api/grading-scales` - List scales
- `PUT /api/grading-scales/:id` - Update (Admin)
- `POST /api/grading-scales/:id/activate` - Set as active (Admin)

### 4. Score Entry (`scores`)
**Domain**: `src/domain/score.rs`
```rust
struct Score {
    id: ObjectId,
    school_id: ObjectId,
    student_id: ObjectId,
    class_subject_id: ObjectId,
    exam_id: ObjectId,
    assessment_category_id: ObjectId,
    education_year_id: ObjectId,
    score: f64,
    max_score: f64,
    percentage: f64, // Calculated
    remarks: Option<String>,
    entered_by: ObjectId, // Teacher who entered
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_deleted: bool,
}

// Audit log for score changes
struct ScoreAuditLog {
    id: ObjectId,
    score_id: ObjectId,
    old_score: f64,
    new_score: f64,
    changed_by: ObjectId,
    change_reason: Option<String>,
    changed_at: DateTime<Utc>,
}
```

**Validation**:
- Only assigned teacher can enter scores for their subject
- No duplicate entries (student + subject + exam + category)
- Score <= max_score

**Endpoints**:
- `POST /api/scores` - Enter score (Teacher, Staff, Admin)
- `POST /api/scores/bulk` - Bulk score entry (Teacher, Staff, Admin)
- `GET /api/scores` - List scores with filters
- `PUT /api/scores/:id` - Update score (creates audit log)
- `GET /api/scores/student/:student_id/exam/:exam_id` - Student scores for exam

### 5. GPA Calculation (`student_term_results`)
**Domain**: `src/domain/student_term_result.rs`
```rust
struct StudentTermResult {
    id: ObjectId,
    school_id: ObjectId,
    student_id: ObjectId,
    class_id: ObjectId,
    education_year_id: ObjectId,
    term_id: String,
    exam_id: ObjectId,
    subject_results: Vec<SubjectResult>,
    total_score: f64,
    total_max_score: f64,
    average_percentage: f64,
    gpa: f64,
    total_credits: Option<i32>,
    grade: String, // Based on grading scale
    rank_in_class: Option<i32>,
    total_students: Option<i32>,
    calculated_at: DateTime<Utc>,
    is_finalized: bool,
}

struct SubjectResult {
    class_subject_id: ObjectId,
    subject_name: String,
    category_scores: Vec<CategoryScore>,
    weighted_score: f64,
    percentage: f64,
    grade: String,
    credits: Option<i32>,
}

struct CategoryScore {
    assessment_category_id: ObjectId,
    category_name: String,
    score: f64,
    max_score: f64,
    weight_percentage: f64,
}
```

**Calculation Logic**:
1. Fetch all scores for student + exam
2. Group by class_subject_id
3. Calculate weighted average per subject using assessment category weights
4. Calculate overall GPA (credit-based or simple average)
5. Apply grading scale to determine letter grade
6. Store in derived collection for fast retrieval

**Service**: `src/services/gpa_calculation_service.rs`
- Uses MongoDB aggregation pipelines
- Triggered after exam completion or on-demand
- Batch processing for all students in a class

**Endpoints**:
- `POST /api/results/calculate/:exam_id` - Calculate results for exam (Admin, Staff)
- `GET /api/results/student/:student_id/term/:term_id` - Get student term results
- `GET /api/results/class/:class_id/exam/:exam_id` - Get class results

### 6. Ranking System (`rankings`)
**Domain**: Embedded in `StudentTermResult`
```rust
struct ClassRanking {
    id: ObjectId,
    school_id: ObjectId,
    class_id: ObjectId,
    exam_id: ObjectId,
    education_year_id: ObjectId,
    rankings: Vec<StudentRank>,
    calculated_at: DateTime<Utc>,
}

struct StudentRank {
    student_id: ObjectId,
    student_name: String,
    gpa: f64,
    average_percentage: f64,
    rank: i32,
    total_students: i32,
}
```

**Ranking Logic**:
- Sort by GPA descending
- Handle ties: same GPA = same rank, next rank skips
- Computed after GPA calculation
- Stored to avoid recalculation

**Endpoints**:
- `POST /api/rankings/calculate/:exam_id` - Calculate rankings (Admin, Staff)
- `GET /api/rankings/class/:class_id/exam/:exam_id` - Get class rankings

### 7. Report Card Generation (`reports`)
**Domain**: `src/domain/report_card.rs`
```rust
struct ReportCard {
    id: ObjectId,
    school_id: ObjectId,
    student_id: ObjectId,
    education_year_id: ObjectId,
    term_id: String,
    exam_id: ObjectId,
    student_info: StudentInfo,
    academic_performance: StudentTermResult,
    attendance_summary: AttendanceSummary,
    teacher_remarks: Vec<TeacherRemark>,
    class_teacher_comment: Option<String>,
    principal_comment: Option<String>,
    report_template_id: Option<ObjectId>,
    generated_at: DateTime<Utc>,
    generated_by: ObjectId,
}

struct StudentInfo {
    name: String,
    registration_number: String,
    class_name: String,
    image: Option<String>,
}

struct AttendanceSummary {
    total_days: i32,
    days_present: i32,
    days_absent: i32,
    attendance_percentage: f64,
}

struct TeacherRemark {
    class_subject_id: ObjectId,
    subject_name: String,
    teacher_name: String,
    remark: String,
}
```

**Report Templates**: Stored as JSON layout configurations
```rust
struct ReportTemplate {
    id: ObjectId,
    school_id: ObjectId,
    name: String,
    layout: serde_json::Value, // JSON structure for report layout
    is_default: bool,
    created_at: DateTime<Utc>,
}
```

**Endpoints**:
- `POST /api/reports/generate/:student_id/:exam_id` - Generate report card
- `GET /api/reports/:id` - Get report card
- `GET /api/reports/student/:student_id` - List student reports
- `POST /api/reports/bulk/:exam_id` - Generate reports for all students in exam

### 8. Transcript Generation (`transcripts`)
**Domain**: `src/domain/transcript.rs`
```rust
struct Transcript {
    id: ObjectId,
    school_id: ObjectId,
    student_id: ObjectId,
    student_info: StudentInfo,
    academic_years: Vec<YearlyPerformance>,
    cumulative_gpa: f64,
    final_status: StudentFinalStatus,
    generated_at: DateTime<Utc>,
    generated_by: ObjectId,
}

struct YearlyPerformance {
    education_year_id: ObjectId,
    year_label: String,
    class_name: String,
    term_results: Vec<TermSummary>,
    yearly_gpa: f64,
    yearly_grade: String,
    promotion_status: PromotionStatus,
}

struct TermSummary {
    term_id: String,
    term_name: String,
    gpa: f64,
    subjects: Vec<SubjectGrade>,
}

struct SubjectGrade {
    subject_name: String,
    final_grade: String,
    credits: Option<i32>,
}

enum StudentFinalStatus { Promoted, Repeated, Graduated, Left }
enum PromotionStatus { Promoted, Repeated, Pending }
```

**Endpoints**:
- `POST /api/transcripts/generate/:student_id` - Generate transcript (Admin, Staff)
- `GET /api/transcripts/:id` - Get transcript (Admin, Staff, Student-own)
- `GET /api/transcripts/student/:student_id` - Get student transcript

### 9. Promotion/Demotion Workflow (`promotions`)
**Domain**: `src/domain/promotion.rs`
```rust
struct PromotionRule {
    id: ObjectId,
    school_id: ObjectId,
    education_year_id: ObjectId,
    name: String,
    min_gpa_threshold: f64,
    min_attendance_percentage: Option<f64>,
    required_subjects_passed: Option<Vec<ObjectId>>,
    custom_rules: Option<serde_json::Value>,
    created_by: ObjectId,
    created_at: DateTime<Utc>,
}

struct PromotionBatch {
    id: ObjectId,
    school_id: ObjectId,
    education_year_id: ObjectId,
    from_class_id: ObjectId,
    to_class_id: Option<ObjectId>,
    promotion_results: Vec<PromotionResult>,
    executed_by: ObjectId,
    executed_at: DateTime<Utc>,
}

struct PromotionResult {
    student_id: ObjectId,
    student_name: String,
    current_gpa: f64,
    promotion_status: PromotionStatus,
    reason: String,
    promoted_to_class_id: Option<ObjectId>,
}
```

**Promotion Engine Logic**:
1. Fetch all students in class
2. Get their final GPA for the year
3. Apply promotion rules
4. Determine: Promote, Repeat, or Graduate
5. Update student.class_id if promoted
6. Archive academic year records
7. Create audit log

**Endpoints**:
- `POST /api/promotions/rules` - Create promotion rule (Admin)
- `GET /api/promotions/rules` - List rules
- `POST /api/promotions/evaluate/:education_year_id` - Evaluate promotions (Admin)
- `POST /api/promotions/execute/:batch_id` - Execute promotion batch (Admin)
- `GET /api/promotions/preview/:education_year_id/:class_id` - Preview promotion results

### 10. Performance Analytics (`analytics`)
**Endpoints**:
- `GET /api/analytics/class/:class_id/exam/:exam_id/distribution` - Score distribution
- `GET /api/analytics/subject/:subject_id/pass-rate` - Subject pass/fail rate
- `GET /api/analytics/class/:class_id/top-students` - Top 10 students
- `GET /api/analytics/class/:class_id/at-risk` - Students below threshold
- `GET /api/analytics/school/performance-trends` - School-wide trends

**Aggregation Pipelines**: All analytics use MongoDB aggregation for performance

## Database Indexes

```javascript
// exams collection
db.exams.createIndex({ school_id: 1, education_year_id: 1, status: 1 })
db.exams.createIndex({ school_id: 1, class_id: 1, start_date: -1 })

// assessment_categories collection
db.assessment_categories.createIndex({ school_id: 1, class_subject_id: 1 })

// scores collection
db.scores.createIndex({ school_id: 1, student_id: 1, exam_id: 1 })
db.scores.createIndex({ school_id: 1, class_subject_id: 1, exam_id: 1 })
db.scores.createIndex({ student_id: 1, class_subject_id: 1, exam_id: 1, assessment_category_id: 1 }, { unique: true })

// student_term_results collection
db.student_term_results.createIndex({ school_id: 1, student_id: 1, education_year_id: 1 })
db.student_term_results.createIndex({ school_id: 1, class_id: 1, exam_id: 1 })

// rankings collection
db.rankings.createIndex({ school_id: 1, class_id: 1, exam_id: 1 })
```

## Role-Based Access Control

| Endpoint | Admin | Staff | Teacher | Student |
|----------|-------|-------|---------|---------|
| Create Exam | ✓ | ✓ | ✗ | ✗ |
| View Exams | ✓ | ✓ | ✓ (own classes) | ✓ (own) |
| Enter Scores | ✓ | ✓ | ✓ (own subjects) | ✗ |
| Calculate GPA | ✓ | ✓ | ✗ | ✗ |
| Generate Reports | ✓ | ✓ | ✓ (own students) | ✓ (own) |
| Execute Promotions | ✓ | ✗ | ✗ | ✗ |
| View Analytics | ✓ | ✓ | ✓ (own classes) | ✗ |

## Implementation Order

1. **Phase 1**: Domain models + Exams + Assessment Categories
2. **Phase 2**: Grading Scales + Score Entry
3. **Phase 3**: GPA Calculation Service + Rankings
4. **Phase 4**: Report Cards + Transcripts
5. **Phase 5**: Promotion Engine
6. **Phase 6**: Analytics + Performance Metrics

## Testing Strategy

- Unit tests for GPA calculation logic
- Unit tests for ranking algorithm (including tie-breaking)
- Integration tests for score entry validation
- Integration tests for promotion workflow
- Load tests for aggregation pipelines

## Migration Notes

- All collections use soft deletes (`is_deleted: bool`)
- Audit logging for all score modifications
- Backward compatible with existing modules
- No breaking changes to existing APIs
