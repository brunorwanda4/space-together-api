# Data Backup & Restore System - Implementation Guide

## Overview

This document describes the complete implementation of the enterprise-grade backup, restore, and soft delete system for the school management platform.

## 🎯 Features Implemented

1. **Manual Backup System** - Admin-triggered backups per school
2. **Automated Backup System** - Scheduled daily backups (ready for cron integration)
3. **Restore Mechanism** - Safe school database restoration
4. **Soft Delete System** - Recoverable deletion with audit trail
5. **Recycle Bin** - Global view of deleted entities
6. **Audit Logging** - Complete tracking of all backup/restore/delete operations

---

## 📁 Files Created

### Domain Layer
- `src/domain/backup.rs` - Backup entity schema with relations

### Service Layer
- `src/services/backup_service.rs` - Backup/restore business logic
- `src/services/recycle_bin_service.rs` - Soft delete management

### Pipeline Layer
- `src/pipeline/backup_pipeline.rs` - Backup aggregation pipeline

### API Layer
- `src/api/backups_api.rs` - Backup REST endpoints
- `src/api/recycle_bin_api.rs` - Recycle bin REST endpoints

### Repository Layer
- Updated `src/repositories/base_repo.rs` - Added `update_one_raw` method

---

## 🔐 Security & Isolation

### School Isolation
- All backups are scoped to `school_id`
- Cross-school restore is prevented
- Backup files are isolated per school

### Permission Requirements
- **Manual Backup**: ADMIN only
- **Restore Backup**: ADMIN only
- **View Recycle Bin**: ADMIN only
- **Restore Entity**: ADMIN or SCHOOLSTAFF
- **Soft Delete**: ADMIN, SCHOOLSTAFF, or TEACHER (entity-specific)

---

## 📡 API Endpoints

### Backup Endpoints

#### 1. Get All Backups
```http
GET /backups
GET /api/backups
```

Query Parameters:
- `filter` - Search term
- `limit` - Results per page (default: 50)
- `skip` - Pagination offset
- `school_id` - Filter by school
- `status` - Filter by status (COMPLETED, FAILED, IN_PROGRESS)
- `backup_type` - Filter by type (MANUAL, AUTOMATED)

Response:
```json
{
  "data": [
    {
      "id": "507f1f77bcf86cd799439011",
      "school_id": "507f1f77bcf86cd799439012",
      "backup_name": "manual_backup_507f1f77bcf86cd799439012_20240217_143022",
      "backup_type": "MANUAL",
      "file_path": "backups/manual_backup_507f1f77bcf86cd799439012_20240217_143022.archive",
      "size_bytes": 1048576,
      "status": "COMPLETED",
      "created_by": "507f1f77bcf86cd799439013",
      "created_at": "2024-02-17T14:30:22Z",
      "completed_at": "2024-02-17T14:32:15Z",
      "error_message": null
    }
  ],
  "total": 10,
  "total_pages": 1,
  "current_page": 1
}
```

#### 2. Get Backups with Relations
```http
GET /backups/others
GET /api/backups/others
```

Includes populated `school` and `creator` objects.

#### 3. Get Single Backup
```http
GET /backups/{id}
GET /api/backups/{id}
```

#### 4. Get Single Backup with Relations
```http
GET /backups/{id}/others
GET /api/backups/{id}/others
```

#### 5. Create Manual Backup
```http
POST /backups/manual
POST /api/backups/manual
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Response:
```json
{
  "id": "507f1f77bcf86cd799439011",
  "school_id": "507f1f77bcf86cd799439012",
  "backup_name": "manual_backup_507f1f77bcf86cd799439012_20240217_143022",
  "backup_type": "MANUAL",
  "file_path": "backups/manual_backup_507f1f77bcf86cd799439012_20240217_143022.archive",
  "size_bytes": 0,
  "status": "IN_PROGRESS",
  "created_by": "507f1f77bcf86cd799439013",
  "created_at": "2024-02-17T14:30:22Z",
  "completed_at": null,
  "error_message": null
}
```

Note: Backup runs asynchronously. Poll the backup status to check completion.

#### 6. Restore Backup
```http
POST /backups/{id}/restore
POST /api/backups/{id}/restore
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Response:
```json
{
  "id": "507f1f77bcf86cd799439014",
  "school_id": "507f1f77bcf86cd799439012",
  "backup_name": "restore_manual_backup_507f1f77bcf86cd799439012_20240217_143022",
  "status": "IN_PROGRESS",
  "created_at": "2024-02-17T15:00:00Z"
}
```

Security:
- Validates backup belongs to user's school
- Prevents restore if another restore is in progress
- Only restores COMPLETED backups
- Validates backup file exists

#### 7. Delete Backup
```http
DELETE /backups/{id}
DELETE /api/backups/{id}
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Deletes both the backup record and the physical backup file.

#### 8. Count Backups
```http
GET /backups/count
GET /api/backups/count
```

Query Parameters: Same as Get All Backups

Response:
```json
{
  "count": 10
}
```

---

### Recycle Bin Endpoints

#### 1. Get Deleted Entities
```http
GET /recycle-bin
GET /api/recycle-bin
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Query Parameters:
- `entity_type` - Filter by entity type (students, teachers, classes, etc.)
- `start_date` - Filter from date (ISO 8601 format)
- `end_date` - Filter to date (ISO 8601 format)
- `limit` - Results per page (default: 20)
- `skip` - Pagination offset

Response:
```json
{
  "data": [
    {
      "entity_type": "students",
      "entity_id": "507f1f77bcf86cd799439011",
      "entity_name": "John Doe",
      "deleted_at": "2024-02-17T14:30:22Z",
      "deleted_by": "507f1f77bcf86cd799439013",
      "deleted_by_name": null,
      "school_id": "507f1f77bcf86cd799439012"
    }
  ],
  "total": 5,
  "total_pages": 1,
  "current_page": 1
}
```

#### 2. Restore Entity
```http
POST /recycle-bin/restore
POST /api/recycle-bin/restore
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Request Body:
```json
{
  "entity_type": "students",
  "entity_id": "507f1f77bcf86cd799439011"
}
```

Response:
```json
{
  "message": "Entity restored successfully"
}
```

#### 3. Permanently Delete Entity
```http
DELETE /recycle-bin/permanent
DELETE /api/recycle-bin/permanent
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Request Body:
```json
{
  "entity_type": "students",
  "entity_id": "507f1f77bcf86cd799439011"
}
```

Response:
```json
{
  "message": "Entity permanently deleted"
}
```

---

### Soft Delete Endpoints (Example: Students)

#### Soft Delete Student
```http
DELETE /students/{id}
DELETE /api/students/{id}
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Sets `deleted_at` and `deleted_by` fields instead of removing the document.

#### Restore Student
```http
POST /students/{id}/restore
POST /api/students/{id}/restore
```

Headers:
```
Authorization: Bearer <jwt_token>
```

Removes `deleted_at` and `deleted_by` fields to restore the student.

---

## 🗄️ Database Schema

### SchoolBackup Collection

```javascript
{
  _id: ObjectId,
  school_id: ObjectId,
  backup_name: String,
  backup_type: "MANUAL" | "AUTOMATED",
  file_path: String,
  size_bytes: Number,
  status: "IN_PROGRESS" | "COMPLETED" | "FAILED",
  created_by: ObjectId,
  created_at: Date,
  completed_at: Date,
  error_message: String
}
```

Indexes:
- `school_id` (non-unique)
- `status` (non-unique)
- `backup_type` (non-unique)
- `created_by` (non-unique)
- `school_id + created_at` (compound, descending)
- `school_id + status` (compound)

### Soft Delete Fields (Added to Entities)

```javascript
{
  // ... existing fields ...
  deleted_at: Date,      // null if not deleted
  deleted_by: ObjectId   // user who deleted
}
```

Collections with Soft Delete:
- students
- teachers
- school_staff
- parents
- classes
- announcements
- assignments
- exams

---

## 🔄 Backup Process Flow

### Manual Backup

1. Admin triggers backup via `POST /backups/manual`
2. System creates backup record with status `IN_PROGRESS`
3. Audit log entry created: `backup.manual.create`
4. Background task spawned to perform backup
5. `mongodump` executed for school database
6. Backup file saved to `backups/` directory with gzip compression
7. Backup record updated with:
   - `status`: `COMPLETED` or `FAILED`
   - `size_bytes`: File size
   - `completed_at`: Completion timestamp
   - `error_message`: Error details if failed

### Automated Backup (Ready for Cron)

To implement automated backups, create a scheduled task that calls:

```rust
// Pseudo-code for cron job
async fn automated_backup_job(state: &AppState) {
    let school_service = SchoolService::new(&state.db.main_db());
    let schools = school_service.get_all_schools().await?;
    
    for school in schools {
        let backup_service = BackupService::new(&state.db.main_db());
        
        // Create automated backup
        let backup = SchoolBackup {
            school_id: school.id,
            backup_type: BackupType::AUTOMATED,
            // ... other fields
        };
        
        backup_service.create_automated_backup(backup).await?;
    }
}
```

Schedule this to run daily using:
- Linux: cron
- Windows: Task Scheduler
- Kubernetes: CronJob
- Cloud: AWS EventBridge, Azure Functions Timer

---

## 🔄 Restore Process Flow

1. Admin triggers restore via `POST /backups/{id}/restore`
2. System validates:
   - Backup belongs to user's school
   - Backup status is `COMPLETED`
   - Backup file exists
   - No other restore in progress
3. Restore record created with status `IN_PROGRESS`
4. Audit log entry created: `backup.restore`
5. Background task spawned to perform restore
6. `mongorestore` executed with `--drop` flag
7. School database restored from backup file
8. Indexes automatically rebuilt
9. Restore record updated with completion status

---

## 🗑️ Soft Delete Process Flow

### Delete Operation

1. User triggers delete (e.g., `DELETE /students/{id}`)
2. System sets:
   - `deleted_at`: Current timestamp
   - `deleted_by`: User ID
3. Document remains in database
4. Audit log entry created: `student.delete`
5. All queries automatically filter `deleted_at: null`

### Restore Operation

1. Admin triggers restore (e.g., `POST /students/{id}/restore`)
2. System removes:
   - `deleted_at` field
   - `deleted_by` field
3. Document becomes visible again
4. Audit log entry created: `student.restore`

### Permanent Delete

1. Admin triggers permanent delete via recycle bin
2. System validates entity is soft-deleted
3. Document permanently removed from database
4. Audit log entry created: `student.permanent_delete`

---

## 📊 Audit Log Events

All backup/restore/delete operations are logged:

### Backup Events
- `backup.manual.create` - Manual backup created
- `backup.automated.create` - Automated backup created
- `backup.restore` - Backup restored
- `backup.delete` - Backup deleted

### Entity Events
- `{entity}.delete` - Entity soft deleted
- `{entity}.restore` - Entity restored from soft delete
- `{entity}.permanent_delete` - Entity permanently deleted

Example audit log entry:
```json
{
  "school_id": "507f1f77bcf86cd799439012",
  "user_id": "507f1f77bcf86cd799439013",
  "user_role": "ADMIN",
  "action": "backup.manual.create",
  "entity_type": "backup",
  "entity_id": "507f1f77bcf86cd799439011",
  "metadata": {
    "backup_name": "manual_backup_507f1f77bcf86cd799439012_20240217_143022",
    "file_path": "backups/manual_backup_507f1f77bcf86cd799439012_20240217_143022.archive"
  },
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0...",
  "severity": "INFO",
  "created_at": "2024-02-17T14:30:22Z"
}
```

---

## 🛠️ Frontend Integration Guide

### 1. Backup Management Page

#### List Backups
```typescript
async function getBackups(page: number = 1, limit: number = 20) {
  const skip = (page - 1) * limit;
  const response = await fetch(
    `/api/backups/others?limit=${limit}&skip=${skip}`,
    {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    }
  );
  return await response.json();
}
```

#### Create Manual Backup
```typescript
async function createBackup() {
  const response = await fetch('/api/backups/manual', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  });
  
  const backup = await response.json();
  
  // Poll for completion
  const checkStatus = setInterval(async () => {
    const status = await fetch(`/api/backups/${backup.id}`);
    const data = await status.json();
    
    if (data.status === 'COMPLETED' || data.status === 'FAILED') {
      clearInterval(checkStatus);
      // Update UI
    }
  }, 5000); // Check every 5 seconds
}
```

#### Restore Backup
```typescript
async function restoreBackup(backupId: string) {
  // Show confirmation dialog
  const confirmed = confirm(
    'Are you sure you want to restore this backup? This will replace all current data.'
  );
  
  if (!confirmed) return;
  
  const response = await fetch(`/api/backups/${backupId}/restore`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  const restore = await response.json();
  
  // Poll for completion
  const checkStatus = setInterval(async () => {
    const status = await fetch(`/api/backups/${restore.id}`);
    const data = await status.json();
    
    if (data.status === 'COMPLETED' || data.status === 'FAILED') {
      clearInterval(checkStatus);
      // Update UI or reload application
    }
  }, 5000);
}
```

### 2. Recycle Bin Page

#### List Deleted Entities
```typescript
async function getDeletedEntities(
  entityType?: string,
  startDate?: string,
  endDate?: string,
  page: number = 1
) {
  const params = new URLSearchParams({
    limit: '20',
    skip: String((page - 1) * 20)
  });
  
  if (entityType) params.append('entity_type', entityType);
  if (startDate) params.append('start_date', startDate);
  if (endDate) params.append('end_date', endDate);
  
  const response = await fetch(`/api/recycle-bin?${params}`, {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  return await response.json();
}
```

#### Restore Entity
```typescript
async function restoreEntity(entityType: string, entityId: string) {
  const response = await fetch('/api/recycle-bin/restore', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      entity_type: entityType,
      entity_id: entityId
    })
  });
  
  if (response.ok) {
    // Refresh list
    await getDeletedEntities();
  }
}
```

#### Permanently Delete
```typescript
async function permanentlyDelete(entityType: string, entityId: string) {
  const confirmed = confirm(
    'Are you sure? This action cannot be undone!'
  );
  
  if (!confirmed) return;
  
  const response = await fetch('/api/recycle-bin/permanent', {
    method: 'DELETE',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      entity_type: entityType,
      entity_id: entityId
    })
  });
  
  if (response.ok) {
    // Refresh list
    await getDeletedEntities();
  }
}
```

### 3. Entity Management (Example: Students)

#### Soft Delete Student
```typescript
async function deleteStudent(studentId: string) {
  const response = await fetch(`/api/students/${studentId}`, {
    method: 'DELETE',
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  if (response.ok) {
    // Show success message
    // Refresh student list
  }
}
```

#### Restore Student
```typescript
async function restoreStudent(studentId: string) {
  const response = await fetch(`/api/students/${studentId}/restore`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  if (response.ok) {
    // Show success message
    // Refresh student list
  }
}
```

### 4. UI Components

#### Backup Status Badge
```typescript
function BackupStatusBadge({ status }: { status: string }) {
  const colors = {
    'IN_PROGRESS': 'blue',
    'COMPLETED': 'green',
    'FAILED': 'red'
  };
  
  return (
    <span className={`badge badge-${colors[status]}`}>
      {status}
    </span>
  );
}
```

#### Backup Size Formatter
```typescript
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}
```

---

## ⚙️ Configuration

### Environment Variables

```env
# MongoDB Connection
MONGODB_URI=mongodb://localhost:27017

# Backup Storage
BACKUP_DIR=./backups

# Optional: S3 Configuration (for future enhancement)
# AWS_ACCESS_KEY_ID=your_key
# AWS_SECRET_ACCESS_KEY=your_secret
# AWS_S3_BUCKET=your_bucket
# AWS_REGION=us-east-1
```

### Prerequisites

1. **MongoDB Tools**: Install `mongodump` and `mongorestore`
   ```bash
   # Ubuntu/Debian
   sudo apt-get install mongodb-database-tools
   
   # macOS
   brew install mongodb-database-tools
   
   # Windows
   # Download from https://www.mongodb.com/try/download/database-tools
   ```

2. **Backup Directory**: Ensure writable directory exists
   ```bash
   mkdir -p backups
   chmod 755 backups
   ```

---

## 🚀 Deployment Checklist

- [ ] Install MongoDB database tools on server
- [ ] Create backup directory with proper permissions
- [ ] Configure environment variables
- [ ] Set up automated backup cron job
- [ ] Configure backup retention policy
- [ ] Set up monitoring for backup failures
- [ ] Test restore process in staging environment
- [ ] Document backup/restore procedures for ops team
- [ ] Set up alerts for failed backups
- [ ] Configure backup file cleanup (old backups)

---

## 📈 Performance Considerations

1. **Async Operations**: All backup/restore operations run in background
2. **Compression**: Backups use gzip compression to reduce file size
3. **Pagination**: All list endpoints support pagination
4. **Indexes**: Proper indexes on backup collection for fast queries
5. **Soft Delete**: Minimal performance impact with proper indexing

---

## 🔒 Security Best Practices

1. **Access Control**: Only ADMIN can perform backup/restore operations
2. **School Isolation**: Strict validation prevents cross-school access
3. **Audit Trail**: All operations logged for compliance
4. **File Validation**: Backup file existence checked before restore
5. **Concurrent Protection**: Prevents multiple simultaneous restores

---

## 🐛 Troubleshooting

### Backup Fails

1. Check MongoDB tools are installed
2. Verify MongoDB connection string
3. Check backup directory permissions
4. Review error_message in backup record
5. Check disk space

### Restore Fails

1. Verify backup file exists
2. Check backup status is COMPLETED
3. Ensure no other restore in progress
4. Verify MongoDB connection
5. Check user permissions

### Soft Delete Not Working

1. Verify entity schema has deleted_at field
2. Check pipeline filters deleted_at: null
3. Verify service uses soft delete method
4. Check audit logs for delete events

---

## 📝 Summary

The backup and restore system provides enterprise-grade data protection with:

- ✅ Manual and automated backups per school
- ✅ Safe restore with validation and safeguards
- ✅ Soft delete with recovery capability
- ✅ Global recycle bin for deleted entities
- ✅ Complete audit trail
- ✅ School isolation and security
- ✅ Async operations for performance
- ✅ RESTful API for frontend integration

All operations follow the existing architecture pattern (Domain → Service → Pipeline → API) and maintain consistency with the codebase.
