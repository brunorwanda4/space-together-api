# Enterprise Audit Log System

Before writing any code:

1. Analyze how modules are structured:

   * `student.rs`
   * `student_service.rs`
   * `student_pipeline.rs`
   * `students_api.rs`
2. Follow the same separation.
3. Do NOT change existing API structure.
4. Use `/` style routes only.
5. Do NOT put business logic inside middleware.
6. All audit writes must go through a service layer.
7. All queries must be scoped by `school_id`.

Now implement a global audit logging system.

---

# 🎯 GOAL

Track critical operations across modules:

* Grade updates
* Attendance changes
* Finance transactions
* Role assignments
* Timetable edits
* Feature toggles
* Student/Teacher CRUD
* Assignment grading
* Submission deletion

Audit logs must be:

* Immutable
* Queryable
* Filterable
* School-scoped
* Efficiently indexed

---

# 1️⃣ DOMAIN — audit_log.rs

Create:

```
audit_log.rs
audit_log_service.rs
audit_log_pipeline.rs
audit_logs_api.rs
```

Follow student module naming pattern exactly.

---

## AuditLog Schema

```rust
pub struct AuditLog {
    pub _id: ObjectId,
    pub school_id: ObjectId,
    pub user_id: ObjectId,
    pub user_role: UserRole,

    pub action: String,         // "grade.update"
    pub entity_type: String,    // "submission"
    pub entity_id: ObjectId,

    pub metadata: Option<Document>,  // before/after changes
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,

    pub created_at: DateTime<Utc>,
}
```

Important:

* NO update endpoint
* NO delete endpoint
* Immutable only

---

# 2️⃣ INDEXING (MANDATORY)

Create indexes:

```
school_id
user_id
entity_type
entity_id
action
created_at (descending)
```

Compound index:

```
{ school_id: 1, created_at: -1 }
```

---

# 3️⃣ SERVICE LAYER

In `audit_log_service.rs`:

### Function:

```rust
pub async fn log_event(
    &self,
    school_id: ObjectId,
    user: &AuthUser,
    action: &str,
    entity_type: &str,
    entity_id: ObjectId,
    metadata: Option<Document>,
    request_meta: Option<RequestMeta>,
) -> Result<(), Error>
```

Where `RequestMeta` contains:

* ip_address
* user_agent

Do NOT fail main transaction if audit logging fails.
Log errors silently.

---

# 4️⃣ INTEGRATION INTO EXISTING MODULES

In critical service functions, inject:

Example (grading submission):

```rust
audit_log_service.log_event(
   school_id,
   user,
   "submission.grade.update",
   "submission",
   submission_id,
   Some(doc!{
       "before_score": old_score,
       "after_score": new_score
   }),
   request_meta
).await;
```

Apply to:

* grade update
* attendance delete
* finance update
* timetable edit
* role assignment
* feature toggle
* student delete
* teacher update

DO NOT duplicate logic inside controllers.
Call audit inside service layer after successful operation.

---

# 5️⃣ PIPELINE

Implement:

GET `/audit-logs`

Query params:

```
limit
skip
user_id
entity_type
action
from_date
to_date
```

Only allow:

* ADMIN
* SCHOOLSTAFF
* Or role with `audit.view`

Never allow Student or Parent.

---

# 6️⃣ API

Create:

```
GET /audit-logs
GET /audit-logs/count
```

Response format:

```
{
   data: [...],
   total: number,
   total_pages: number,
   current_page: number
}
```

Follow student list response format exactly.

---

# 7️⃣ SECURITY RULES

* Always filter by school_id
* Never expose metadata of another school
* No update/delete endpoints
* Prevent audit tampering
* Validate date ranges

---

# 8️⃣ PERFORMANCE STRATEGY

* Logs are append-only
* Use projection for list queries
* Allow future archiving strategy
* Use cursor pagination for large data

---

# 9️⃣ OPTIONAL (ADVANCED)

Add:

```
severity: "INFO" | "WARNING" | "CRITICAL"
```

Examples:

* Grade update → INFO
* Finance delete → CRITICAL
* Role removal → WARNING

---

# 🔐 FINAL RESULT

After implementation:

* All critical actions logged
* Logs immutable
* Queryable by admin
* Tenant isolated
* Enterprise compliant

Follow Domain → Service → Pipeline → API strictly.
No architectural deviation.
