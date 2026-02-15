# Assignment & Homework System

Before generating any code:

1. Analyze how existing modules are structured:

   * `student_domain.rs`
   * `student_services.rs`
   * `student_api.rs`
   * `student_pipeline.rs`
2. Follow the same folder structure, naming conventions, DTO separation, validation patterns, role guards, audit logging, and error handling.
3. Respect multi-tenant architecture (database-per-school using X-School-ID header).
4. Reuse existing Cloudinary service for file uploads.
5. Reuse existing EventService for audit logging.
6. Do NOT break existing modules. Create a clean new module.

Now implement a complete Assignment & Homework backend module.

==================================================
MODULE STRUCTURE
================

Create new files:

* `assignment_domain.rs`
* `assignment_dto.rs`
* `assignment_services.rs`
* `assignment_api.rs`
* `assignment_pipeline.rs`

Register routes in main router like other modules.

==================================================
CORE FEATURES TO IMPLEMENT
==========================

1️⃣ Assignment Creation (Teacher)

Teacher can:

* Create assignment per class subject
* Set:

  * title
  * description
  * instructions
  * due_date
  * max_score
  * allow_late_submission (bool)
  * attachment (optional file)
* Only assigned teacher of that subject can create assignment.
* Status: Draft | Published | Archived

Endpoint:
POST /assignments

Guards:

* Teacher role
* Validate teacher is assigned to subject

==================================================
2️⃣ Assignment Listing

Endpoints:

GET /assignments

* Filter by:

  * class_id
  * subject_id
  * teacher_id
  * status
  * due_date
* Paginated
* Use aggregation pipeline

GET /assignments/{id}

* Return with:

  * teacher info
  * subject info
  * submission count
  * total students

==================================================
3️⃣ Student Submission

Students can:

* Submit before due date
* Upload file (PDF, DOC, image)
* Add optional comment
* Only one active submission allowed (can update until deadline)

Endpoint:
POST /assignments/{id}/submit

Rules:

* Validate student belongs to class
* Enforce deadline
* If allow_late_submission == false → reject late submission
* If late allowed → mark submission as "Late"

Store:

Submission:
{
assignment_id,
student_id,
file_url,
file_id,
comment,
submitted_at,
is_late,
score (optional),
feedback (optional),
graded_at (optional),
graded_by (teacher_id),
status: Submitted | Graded
}

==================================================
4️⃣ Deadline Enforcement

* Validate due_date at service layer
* Add index on due_date
* Add helper function:
  validate_submission_deadline()

==================================================
5️⃣ Teacher Grading & Feedback

Teacher can:

* View all submissions per assignment
* Grade submission
* Add feedback comment
* Upload annotated file (optional)
* Change submission status to Graded

Endpoint:
PUT /assignments/{id}/grade/{submission_id}

Rules:

* Only teacher assigned to subject
* Score must be <= max_score
* Audit log grade changes

==================================================
6️⃣ Aggregation Pipelines

Create pipelines:

* assignment_with_teacher_pipeline()
* assignment_submission_summary_pipeline()
* student_submission_pipeline()
* teacher_assignment_dashboard_pipeline()

Return:

For Teacher:

* Total assignments
* Total submissions
* Pending grading count
* Late submissions count

For Student:

* Assigned homework list
* Submission status
* Grade if graded
* Upcoming deadlines

==================================================
7️⃣ File Upload Handling

* Use existing Cloudinary service
* Store:

  * file_url
  * file_id
* Support:

  * Assignment attachment
  * Student submission file
  * Teacher feedback file

==================================================
8️⃣ Role-Based Access Control

Roles:

Teacher:

* Create assignment
* Edit own assignments
* Grade submissions

Student:

* View assigned homework
* Submit homework
* View grade

Admin/Staff:

* View all assignments
* Delete (soft delete)

==================================================
9️⃣ Soft Delete

Assignments and submissions must use:

* is_deleted
* deleted_at

==================================================
🔟 Auto-Grading (Future AI Placeholder)

Add:

* auto_grade_enabled: bool
* auto_grade_score: Option<f64>
* ai_feedback: Option<String>

Create placeholder service method:

async fn trigger_auto_grading()

Do not implement AI logic yet, but structure must support future integration.

==================================================
1️⃣1️⃣ Indexing

Create indexes:

Assignments:

* class_id
* subject_id
* teacher_id
* due_date
* status

Submissions:

* assignment_id
* student_id
* graded_by
* submitted_at

==================================================
1️⃣2️⃣ Audit Logging

Log:

* Assignment created
* Assignment published
* Submission created
* Submission graded
* Submission updated

==================================================
1️⃣3️⃣ Multi-Tenant Safety

* All queries filtered by school_id
* X-School-ID header required
* Validate class and subject belong to school

==================================================
1️⃣4️⃣ Response Structure

Follow existing response wrapper pattern.

Return consistent JSON:

{
data,
message,
total,
total_pages,
current_page
}

==================================================

The system must be:

* Production-ready
* Multi-tenant safe
* Fully async
* Cleanly separated (Domain / DTO / Service / API / Pipeline)
* Consistent with Space-Together architecture

Do not introduce a new pattern. Follow existing codebase strictly.
