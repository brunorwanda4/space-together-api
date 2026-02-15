#Parent / Guardian Portal

Design and implement a complete Parent / Guardian Portal backend module for Space-Together using the same architectural patterns, folder structure, coding conventions, and design principles already used in `student_services.rs`, `student_api.rs`, and `student_pipeline.rs`.

The implementation must follow:

* Rust
* Actix-web
* MongoDB (database-per-school multi-tenancy)
* JWT authentication
* Role-based guards
* Aggregation pipelines
* DTO separation
* Soft delete
* Audit logging
* Async services
* Structured validation
* Consistent error handling

Create new files following existing naming conventions:

* `parent_domain.rs`
* `parent_dto.rs`
* `parent_services.rs`
* `parent_api.rs`
* `parent_pipeline.rs`
* Register routes in main router like other modules

Do NOT modify student module directly — build a clean parent module that integrates with existing collections.

The Parent / Guardian module must include:

1. Parent Account & Authentication

* Allow users with role `Parent` to log in via JWT.
* Parent user must be linked to one or more students via guardian relationship.
* Enforce tenant isolation via X-School-ID header.
* Add guard: `ParentGuard`.

Create relation structure:

* Parent linked to students using student_id reference.
* Support multiple children per parent.

2. Parent Dashboard Endpoint

Create endpoint:

GET /parents/dashboard

Return aggregated data:

* Total children count
* Latest announcements
* Attendance summary per child
* Current term GPA per child
* Outstanding fee balance per child

Use aggregation pipeline inside `parent_pipeline.rs`.

3. Attendance View (Read-Only)

Endpoint:

GET /parents/{student_id}/attendance

Rules:

* Parent can only access attendance of linked students.
* Return:

  * Present/Absent count
  * Attendance percentage
  * Recent attendance records
* Use aggregation pipeline similar to student analytics pattern.

4. Grades & Results View

Endpoint:

GET /parents/{student_id}/results

Return:

* Term GPA
* Rank
* Subject-level breakdown
* Final grades
* Teacher remarks

Reuse existing exam/result collections.
Implement aggregation pipeline inside `parent_pipeline.rs`.

Parent cannot modify any grade.

5. Announcements Access

Endpoint:

GET /parents/announcements

Return:

* School-wide announcements
* Class-specific announcements (if child belongs to class)
* Paginated
* Filter by published status

Reuse announcement collection with filtering logic.

6. Fee Balance & Payment Summary

Endpoint:

GET /parents/{student_id}/finance

Return:

* Total fee required
* Amount paid
* Outstanding balance
* Payment history
* Installments if exist

Use aggregation pipeline across:

* enrollments
* payments
* fee structures

7. Security Rules

* Parent can ONLY access:

  * Their own children
* Validate student ownership before data retrieval.
* Add middleware-level validation helper:
  validate_parent_student_access()

8. Audit Logging

Log:

* Parent login
* Parent dashboard access
* Finance view access

Use existing audit logging infrastructure.

9. Performance

* Use indexes on:

  * student_id
  * parent_id
  * education_year_id
* Use aggregation instead of multiple queries.
* Avoid N+1 queries.

10. Role & Permissions

Add Parent role if not existing in enum.
Update RBAC guards accordingly.

Parent permissions:

* View attendance
* View grades
* View announcements
* View finance
* No write access anywhere

11. Response Structure

All endpoints must:

* Return structured JSON
* Follow existing response wrapper pattern
* Support pagination where applicable
* Use consistent error handling

12. Integration Requirements

* Do not duplicate student logic.
* Reuse:

  * Attendance collection
  * Results collection
  * Finance collection
  * Announcement collection
* Maintain clean separation of concerns.

The module must be production-ready, multi-tenant safe, secure, and consistent with the rest of the Space-Together backend design.
