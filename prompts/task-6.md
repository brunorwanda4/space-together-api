# Data Backup & Restore System

Before generating code:

1. Study how existing modules are structured:

   * `student.rs`
   * `student_service.rs`
   * `student_pipeline.rs`
   * `students_api.rs`
   
2. Follow the same naming and separation pattern.
3. Do NOT introduce a new architecture.
4. Do NOT use `/api/v1`.
5. Keep school isolation strict.
6. Do NOT put business logic in controllers.
7. Use service layer for all backup operations.
8. Integrate audit logging when backup/restore happens.
9. If you need database query [create, update, delete, get] and others use base_repo.rs if that query not exit create it in base_repo.rs and use it
10. after finish make doc file which explain what you did for to implement on fortend

Now implement:

---

# 🎯 GOAL

Enterprise-grade:

1. Automated per-school backup
2. Manual backup trigger
3. Restore mechanism (full school restore)
4. Soft delete system with recovery
5. Audit logging for all backup/restore actions

---

# 1️⃣ BACKUP SYSTEM

Create module:

```
backup.rs
backup_service.rs
backup_pipeline.rs
backups_api.rs
```

Follow student module naming convention exactly.

---

## Backup Schema

```rust
pub struct SchoolBackup {
    pub _id: ObjectId,
    pub school_id: ObjectId,
    pub backup_name: String,
    pub backup_type: String, // "AUTOMATED" | "MANUAL"
    pub file_path: String,
    pub size_bytes: i64,
    pub status: String, // "COMPLETED" | "FAILED" | "IN_PROGRESS"
    pub created_by: Option<ObjectId>,
    pub created_at: DateTime<Utc>,
}
```

Important:

* Each backup belongs to one school only.
* No cross-school restore allowed.

---

## Backup Service Requirements

Implement in `backup_service.rs`:

### 1. create_manual_backup(school_id, user)

* Trigger MongoDB dump for that school database
* Store file in:

  * local storage
  * OR S3-compatible storage (abstract storage layer)
* Record metadata in SchoolBackup collection
* Log audit event:
  "backup.manual.create"

### 2. automated_backup_job()

* Scheduled task (cron)
* Runs daily
* Iterates all school databases
* Creates automated backup
* Type: "AUTOMATED"

Do NOT block main server thread.
Use background task.

---

# 2️⃣ RESTORE SYSTEM

Implement:

```
restore_backup(backup_id, user)
```

Rules:

* Only ADMIN
* Only within same school
* Must confirm restore
* Log audit:
  "backup.restore"

Process:

1. Validate backup belongs to school
2. Drop current school database safely
3. Restore from dump
4. Rebuild indexes
5. Mark restore event in logs

Add safeguard:

* Cannot restore if another restore is running
* Use distributed lock or restore flag

---

# 3️⃣ SOFT DELETE SYSTEM

Implement generic soft delete pattern across modules.

Modify entities:

Add to critical schemas:

```rust
pub deleted_at: Option<DateTime<Utc>>,
pub deleted_by: Option<ObjectId>,
```

Replace delete operations:

Instead of removing document:

```rust
set deleted_at = now
set deleted_by = user_id
```

Modify pipelines:

All normal queries must filter:

```
deleted_at: null
```

---

## Recovery Endpoint

Create generic restore endpoint per module:

Example:

```
POST /students/{id}/restore
```

Rules:

* Only ADMIN or SCHOOLSTAFF
* Only if deleted_at != null
* Log audit:
  "student.restore"

---

# 4️⃣ GLOBAL RECYCLE BIN

Create:

```
GET /recycle-bin
```

Query returns:

* entity_type
* entity_id
* deleted_by
* deleted_at

Filter by:

* entity_type
* date range

Admin only.

---

# 5️⃣ SECURITY RULES

* Always filter by school_id
* Never restore backup from another school
* Prevent double restore
* Prevent restore if backup status != COMPLETED
* Validate backup file exists
* Audit every restore

---

# 6️⃣ INDEXING

Backup collection:

```
school_id
created_at (desc)
status
```

Soft delete:

Add index:

```
deleted_at
```

---

# 7️⃣ AUDIT INTEGRATION

Log:

* Manual backup created
* Automated backup created
* Restore performed
* Entity restored
* Entity soft deleted

---

# 8️⃣ PERFORMANCE STRATEGY

* Backup async only
* Large schools handled in background
* Paginated backup list
* Optionally compress dumps

---

# 🔐 FINAL BACKEND RESULT

After implementation:

* Each school has independent backups
* Admin can trigger manual backup
* System runs automated backups
* Restore safe and isolated
* Soft delete protects data
* Full recovery possible
* Enterprise-level reliability

Follow Domain → Service → Pipeline → API strictly.
Do not change architectural pattern.
