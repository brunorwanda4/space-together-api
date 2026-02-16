# Audit Log Integration Guide

This document shows how to integrate audit logging into existing services for critical operations.

## Overview

The audit log system tracks critical operations across the application:
- Grade updates
- Attendance changes
- Finance transactions
- Role assignments
- Timetable edits
- Feature toggles
- Student/Teacher CRUD
- Assignment grading
- Submission deletion

## Architecture

```
Domain → Service → Pipeline → API
```

All audit logging happens in the SERVICE LAYER, never in controllers or middleware.

## Files Created

1. `src/domain/audit_log.rs` - Domain model
2. `src/services/audit_log_service.rs` - Service layer
3. `src/pipeline/audit_log_pipeline.rs` - Aggregation pipeline
4. `src/api/audit_logs_api.rs` - API endpoints

## API Endpoints

All endpoints require authentication and `audit.view` permission (ADMIN or SCHOOLSTAFF roles).

### GET /audit-logs
List all audit logs with pagination and filters

Query parameters:
- `limit` - Number of records per page (default: 10)
- `skip` - Number of records to skip for pagination
- `filter` - Search text across multiple fields
- `user_id` - Filter by specific user ObjectId
- `entity_type` - Filter by entity type (e.g., "submission", "student")
- `action` - Filter by action (e.g., "submission.grade.update")
- `entity_id` - Filter by specific entity ObjectId
- `severity` - Filter by severity (INFO, WARNING, CRITICAL)
- `from_date` - Filter by date range (created_at >= from_date)
- `to_date` - Filter by date range (created_at <= to_date)

Response:
```json
{
  "data": [...],
  "total": 100,
  "total_pages": 10,
  "current_page": 1
}
```

### GET /audit-logs/others
List audit logs with user and school relations populated

Same query parameters as above. Returns audit logs with nested user and school objects.

### GET /audit-logs/{id}
Get a single audit log by ID

Path parameters:
- `id` - Audit log ObjectId or string ID

### GET /audit-logs/{id}/others
Get a single audit log by ID with relations populated

Path parameters:
- `id` - Audit log ObjectId or string ID

Returns audit log with nested user and school objects.

### GET /audit-logs/match
Find a single audit log by custom match criteria

Query parameters: Same as list endpoints
Returns the first matching audit log.

### GET /audit-logs/others/match
Find a single audit log by custom match criteria with relations

Query parameters: Same as list endpoints
Returns the first matching audit log with nested user and school objects.

### GET /audit-logs/count
Get count of audit logs matching filters

Query parameters:
- `filter` - Search text
- Any filter parameters from the list endpoint

Response:
```json
{
  "count": 42
}
```

## Permission Requirements

Only users with the following roles can view audit logs:
- ADMIN
- SCHOOLSTAFF
- Users with `audit.view` permission

Students and Parents CANNOT view audit logs.

## Integration Examples

### 1. Assignment Grading

Add to `src/services/assignment_service.rs`:

```rust
use crate::domain::audit_log::{AuditSeverity, RequestMeta};
use crate::services::audit_log_service::AuditLogService;

pub async fn grade_submission(
    &self,
    submission_id: &IdType,
    score: f64,
    feedback: Option<String>,
    feedback_file: Option<String>,
    graded_by: mongodb::bson::oid::ObjectId,
    user: &AuthUserDto,
    school_id: ObjectId,
    db: &Database,
    request_meta: Option<RequestMeta>,
) -> Result<Submission, AppError> {
    let submission = self.find_one_submission(Some(submission_id), None).await?;
    let old_score = submission.score;

    // ... existing validation and update logic ...

    let result = self.update_submission(submission_id, &update).await?;

    // Audit log after successful operation
    let audit_service = AuditLogService::new(db);
    let submission_oid = IdType::to_object_id(submission_id)?;
    
    audit_service.log_event(
        school_id,
        user,
        "submission.grade.update",
        "submission",
        submission_oid,
        Some(doc! {
            "before_score": old_score,
            "after_score": score,
            "feedback_provided": feedback.is_some()
        }),
        request_meta,
        Some(AuditSeverity::INFO),
    ).await?;

    Ok(result)
}
```

### 2. Student Deletion

```rust
pub async fn delete(
    &self, 
    id: &IdType,
    user: &AuthUserDto,
    school_id: ObjectId,
    db: &Database,
    request_meta: Option<RequestMeta>,
) -> Result<Student, AppError> {
    let student = self.find_one(Some(id), None).await?;
    
    // ... existing deletion logic ...

    let audit_service = AuditLogService::new(db);
    let student_oid = IdType::to_object_id(id)?;
    
    audit_service.log_event(
        school_id,
        user,
        "student.delete",
        "student",
        student_oid,
        Some(doc! {
            "student_name": student.name.clone(),
            "student_email": student.email.clone()
        }),
        request_meta,
        Some(AuditSeverity::CRITICAL),
    ).await?;

    Ok(student)
}
```

## Action Naming Convention

Use dot notation:
- `entity.operation` - e.g., `student.create`, `student.delete`
- `entity.field.operation` - e.g., `submission.grade.update`

## Severity Levels

- `INFO` - Normal operations
- `WARNING` - Important changes
- `CRITICAL` - High-impact operations

## Security

- Audit logs are immutable (no update/delete endpoints)
- Always filtered by school_id
- Logs never fail main transaction
