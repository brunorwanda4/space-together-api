
# Critical Functional Gaps (Must Have for V1.5)

Design and implement a complete Academic Records & Assessment System for Space-Together, following the same architecture patterns already used in the project (Rust, Actix-web, MongoDB, database-per-school multi-tenancy, JWT auth, role-based guards, aggregation pipelines, async handlers, structured schemas, validation, soft deletes, pagination, filtering, and audit logging).

The implementation must integrate cleanly with existing modules such as Users, Classes, ClassSubjects, EducationYear, and Attendance.

Build the following modules:

1. Exam Management
   Create an Exams module where schools can define exams per education year and optionally per class (e.g., Term 1, Midterm, Final). Exams must support start date, end date, status (Draft, Published, Archived), and linkage to class subjects. Include CRUD endpoints protected by role guards (Admin, Staff). Ensure tenant isolation using the current database resolution strategy (X-School-ID or subdomain).

2. Assessment Categories
   Implement assessment categories per class subject (CAT, Final, Quiz, Assignment, Practical, etc.). Each category must have weight percentage and be configurable per class subject. Validate that total weight per subject does not exceed 100%.

3. Grading System
   Create configurable grading scales per school. Support:

* Letter grading (A–F with min/max score)
* Percentage-based grading
* Competency-based grading (e.g., Excellent, Good, Needs Improvement)

Allow schools to select one grading strategy per education year. Store grade boundaries and ensure grade calculation logic is abstracted via service layer.

4. Score Entry & Continuous Assessment Tracking
   Implement score recording per student per subject per exam per category.
   Ensure:

* Only assigned teachers can input scores for their subjects.
* Validation prevents duplicate entries.
* Editing is audit-logged.
* Support bulk upload endpoint (future CSV import compatible).

5. GPA / Average Calculation Engine
   Create a calculation service that:

* Computes weighted subject averages.
* Computes overall student GPA per term.
* Supports both credit-based and simple average calculation.
* Uses MongoDB aggregation pipelines for performance.

Store computed results in a derived collection (e.g., student_term_results) for fast retrieval.

6. Ranking Logic
   Implement ranking per class and per exam period.
   Ranking must:

* Be computed automatically after exam publication.
* Handle tie-breaking logic.
* Be stored to avoid recalculating on every request.

7. Report Card Generation
   Create a Report module that:

* Aggregates student performance per term.
* Includes subject scores, GPA, rank, attendance summary, teacher remarks.
* Supports customizable report templates (stored as structured JSON layout).
* Generates a structured response ready for PDF generation (PDF generation itself optional for now).

8. Transcript Generation
   Implement cumulative transcript generation across academic years.
   Transcript must:

* Include yearly GPA.
* Show promotion status.
* Display final grade per subject per year.
* Be accessible only by Admin and authorized Staff.

9. Promotion / Demotion Workflow
   Create a Promotion Engine that:

* Evaluates student eligibility based on configurable GPA threshold or pass rules.
* Promotes students to next class level.
* Marks students as Repeated or Graduated.
* Archives previous academic year records.
* Is executed per education year close operation.

10. Student Performance Analytics
    Add aggregation endpoints for:

* Class performance distribution.
* Subject pass/fail rate.
* Top 10 students.
* Risk detection (students below GPA threshold).

Technical Requirements:

* Follow existing folder/module structure conventions.
* Use DTOs for request/response separation.
* Apply schema validation.
* Use soft delete instead of hard delete.
* Add indexing for performance (student_id, class_id, exam_id).
* Ensure all endpoints are multi-tenant safe.
* Integrate audit log entries for grade changes and promotion actions.
* Add role guards consistent with existing authorization system.
* Write unit tests for GPA calculation and ranking logic.
* Use aggregation pipelines instead of in-memory calculations wherever possible.

The system must be production-ready, scalable, and consistent with current Space-Together backend design philosophy.
