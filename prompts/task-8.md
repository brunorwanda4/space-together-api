Excellent. This is a **strategic enterprise feature** for Space-Together.
Analytics makes your platform feel institutional-grade.

Below is a **strict backend + frontend implementation prompt** tailored to your:

* Rust + Actix architecture
* Domain → Service → Pipeline → API structure
* MongoDB (database-per-school)
* Audit + Permission system already implemented
* No `/api/v1` prefix
* Next.js frontend using your `s-t/...` page pattern
* TanStack + shadcn + SWR

---

# ✅ BACKEND PROMPT — Director Analytics Dashboard

Before writing any code:

1. Read how modules are structured:

   * `student.rs`
   * `student_service.rs`
   * `student_pipeline.rs`
   * `students_api.rs`
2. Follow same structure.
3. Respect multi-tenant isolation (`school_id` always required).
4. Use permission guard.
5. Integrate audit log where relevant.
6. Do NOT use `/api/v1`.
7. Use MongoDB aggregation pipelines for all analytics.
8. Optimize with indexes.

---

# 🎯 GOAL

Build analytics endpoints for Director Dashboard:

1. Student enrollment trends
2. Attendance rate %
3. Pass/fail distribution
4. Fee collection summary
5. Teacher workload distribution

Create module:

```
analytics.rs
analytics_service.rs
analytics_pipeline.rs
analytics_api.rs
```

---

# 🔐 PERMISSIONS

Create new permission namespace:

```
analytics.read.school
```

Only:

* Director
* Admin
* Accountant (limited)
* Vice President (optional)

Teachers/students/parents → denied.

---

# 1️⃣ Student Enrollment Trends

## Objective:

Monthly student registration growth.

## Pipeline logic:

Group students by:

```
$group by year + month (created_at)
count
```

Return:

```json
[
  { "month": "2026-01", "total": 120 },
  { "month": "2026-02", "total": 138 }
]
```

Filter:

* school_id
* deleted_at = null

Endpoint:

```
GET /analytics/enrollment-trends
```

Optional query:

```
?year=2026
```

---

# 2️⃣ Attendance Rate %

Calculate:

```
(total present / total attendance records) * 100
```

Pipeline:

* Match by school_id
* Optional date range
* Group:
  present count
  total count

Return:

```json
{
  "attendance_rate": 87.3
}
```

Endpoint:

```
GET /analytics/attendance-rate
```

Optional:

```
?from=2026-01-01&to=2026-12-31
```

---

# 3️⃣ Pass / Fail Distribution

Assume grades exist.

Group by:

```
status: PASS | FAIL
```

Based on:

```
score >= passing_mark
```

Return:

```json
{
  "pass": 340,
  "fail": 42
}
```

Endpoint:

```
GET /analytics/pass-fail-distribution
```

---

# 4️⃣ Fee Collection Summary

Use finance collection data.

Compute:

* Total expected
* Total paid
* Total unpaid
* Percentage collected

Return:

```json
{
  "total_expected": 12000000,
  "total_collected": 9500000,
  "total_outstanding": 2500000,
  "collection_rate": 79.2
}
```

Endpoint:

```
GET /analytics/fee-summary
```

---

# 5️⃣ Teacher Workload Distribution

Calculate per teacher:

* Number of classes
* Number of subjects
* Total students across classes

Return:

```json
[
  {
    "teacher_id": "...",
    "teacher_name": "John Doe",
    "classes": 4,
    "subjects": 6,
    "total_students": 140
  }
]
```

Endpoint:

```
GET /analytics/teacher-workload
```

---

# 📊 PERFORMANCE REQUIREMENTS

* All queries use aggregation pipelines
* Use indexes:

  * school_id
  * created_at
  * class_id
  * teacher_id
* Cache results (optional future Redis)
* Response under 500ms for 5k students

---

# 🧱 SERVICE LAYER

Analytics service:

* Validate permission
* Call pipeline functions
* Format clean DTO
* No business logic inside API

---

# 📌 ROUTES STRUCTURE

Inside `analytics_api.rs`:

```
GET /analytics/enrollment-trends
GET /analytics/attendance-rate
GET /analytics/pass-fail-distribution
GET /analytics/fee-summary
GET /analytics/teacher-workload
```

No `/api/v1`.

---

# 🎓 EXPECTED BACKEND RESULT

Director can retrieve:

* Growth metrics
* Attendance %
* Academic performance
* Financial health
* Staff distribution

Secure and school-scoped.

This transforms Space-Together from management tool → decision intelligence system.
