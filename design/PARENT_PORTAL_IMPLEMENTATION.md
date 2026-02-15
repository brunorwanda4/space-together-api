# Parent/Guardian Portal Implementation

## Overview

This document describes the complete implementation of the Parent/Guardian Portal backend module for Space-Together. The module follows the same architectural patterns, folder structure, coding conventions, and design principles used throughout the application.

## Architecture

### Technology Stack
- **Language**: Rust
- **Web Framework**: Actix-web
- **Database**: MongoDB (database-per-school multi-tenancy)
- **Authentication**: JWT
- **Authorization**: Role-based guards
- **Data Processing**: Aggregation pipelines
- **Validation**: Structured validation with custom validators

### Design Principles
- DTO separation for request/response handling
- Soft delete support
- Audit logging for sensitive operations
- Async services throughout
- Consistent error handling
- Multi-tenant isolation via X-School-ID header

## Module Structure

### Files Created

1. **Domain Layer** (`src/domain/parent.rs`)
   - `Parent` - Main parent entity
   - `ParentPartial` - Partial update DTO
   - `ParentWithRelations` - Parent with populated relationships
   - `ParentDashboard` - Dashboard response DTO
   - `ChildSummary` - Summary data for each child
   - `AttendanceSummary` - Attendance statistics
   - `StudentResults` - Academic results view
   - `FinanceSummary` - Fee and payment information

2. **Pipeline Layer** (`src/pipeline/parent_pipeline.rs`)
   - `parent_pipeline()` - Main aggregation for parent with relations
   - `parent_dashboard_pipeline()` - Dashboard data aggregation
   - `attendance_summary_pipeline()` - Attendance statistics
   - `student_results_pipeline()` - Academic results aggregation
   - `finance_summary_pipeline()` - Financial data aggregation

3. **Service Layer** (`src/services/parent_service.rs`)
   - CRUD operations (create, read, update, delete)
   - `get_dashboard()` - Parent dashboard with aggregated data
   - `get_attendance_summary()` - Student attendance view
   - `get_student_results()` - Academic results view
   - `get_finance_summary()` - Fee balance and payments
   - `get_parent_announcements()` - Filtered announcements
   - `validate_parent_student_access()` - Access control helper

4. **API Layer** (`src/api/parent_api.rs`)
   - Admin/Staff endpoints for parent management
   - Parent portal endpoints for self-service access

5. **Guards** (`src/guards/role_guard.rs`)
   - `check_parent()` - Verify parent role
   - `check_parent_access()` - Validate parent-student relationship

## Database Schema

### Parents Collection

```javascript
{
  _id: ObjectId,
  user_id: ObjectId,           // Link to users collection
  school_id: ObjectId,          // School association
  student_ids: [ObjectId],      // Array of linked students
  name: String,
  email: String,
  phone: String?,
  gender: Gender?,
  image: String?,
  image_id: String?,
  relationship: String?,        // "Father", "Mother", "Guardian"
  occupation: String?,
  national_id: String?,
  status: ParentStatus,         // Active, Inactive
  is_active: Boolean,
  created_at: DateTime,
  updated_at: DateTime
}
```

### Indexes
- `email` (unique)
- `user_id` (unique, partial)
- `school_id`
- `student_ids`
- `status`
- `is_active`
- Compound: `(school_id, status)`

## API Endpoints

### Admin/Staff Endpoints (Protected)

All require JWT authentication and Admin/Staff role.

#### GET `/parents`
Get all parents with pagination and filtering.

**Query Parameters:**
- `filter` - Search term
- `limit` - Results per page
- `skip` - Pagination offset
- `field[]` - Filter fields
- `value[]` - Filter values

**Response:**
```json
{
  "data": [Parent],
  "total": 100,
  "total_pages": 10,
  "current_page": 1
}
```

#### GET `/parents/others`
Get all parents with populated relationships.

#### GET `/parents/{id}`
Get parent by ID.

#### GET `/parents/{id}/others`
Get parent by ID with relationships.

#### POST `/parents`
Create new parent.

**Request Body:**
```json
{
  "name": "John Doe",
  "email": "john@example.com",
  "phone": "+250788123456",
  "school_id": "...",
  "student_ids": ["...", "..."],
  "relationship": "Father"
}
```

#### PUT `/parents/{id}`
Update parent information.

#### DELETE `/parents/{id}`
Delete parent (soft delete).

#### GET `/parents/count`
Get total parent count with filters.

### Parent Portal Endpoints (Protected)

All require JWT authentication and Parent role.

#### GET `/parents/dashboard`
Get parent dashboard with aggregated data.

**Headers:**
- `X-School-ID` - Required for tenant isolation

**Response:**
```json
{
  "total_children": 2,
  "latest_announcements": [AnnouncementWithRelations],
  "children_summary": [
    {
      "student_id": "...",
      "student_name": "Jane Doe",
      "class_name": "Grade 10A",
      "attendance_percentage": 95.5,
      "current_term_gpa": 3.8,
      "outstanding_fees": 50000
    }
  ]
}
```

#### GET `/parents/{student_id}/attendance`
Get attendance summary for a specific child.

**Validation:** Parent must be linked to the student.

**Response:**
```json
{
  "present_count": 85,
  "absent_count": 5,
  "late_count": 3,
  "excused_count": 2,
  "total_days": 95,
  "attendance_percentage": 89.5,
  "recent_records": [
    {
      "date": "2024-03-15T08:00:00Z",
      "status": "Present",
      "remarks": null
    }
  ]
}
```

#### GET `/parents/{student_id}/results`
Get academic results for a specific child.

**Query Parameters:**
- `education_year_id` - Optional filter
- `term_id` - Optional filter

**Validation:** Parent must be linked to the student.

**Response:**
```json
{
  "term_gpa": 3.8,
  "rank": 5,
  "total_students": 45,
  "grade": "A",
  "subject_results": [
    {
      "subject_name": "Mathematics",
      "score": 85,
      "max_score": 100,
      "percentage": 85.0,
      "grade": "A"
    }
  ],
  "teacher_remarks": "Excellent performance"
}
```

#### GET `/parents/{student_id}/finance`
Get fee balance and payment history for a specific child.

**Validation:** Parent must be linked to the student.

**Response:**
```json
{
  "total_fee_required": 500000,
  "amount_paid": 300000,
  "outstanding_balance": 200000,
  "payment_history": [
    {
      "date": "2024-01-15T10:00:00Z",
      "amount": 150000,
      "payment_method": "Bank Transfer",
      "reference": "TXN123456"
    }
  ],
  "installments": [
    {
      "due_date": "2024-04-01T00:00:00Z",
      "amount": 100000,
      "status": "Pending"
    }
  ]
}
```

#### GET `/parents/announcements`
Get announcements accessible to parent (school-wide and class-specific).

**Query Parameters:**
- `limit` - Results per page
- `skip` - Pagination offset

**Response:**
```json
{
  "data": [AnnouncementWithRelations],
  "total": 15,
  "total_pages": 2,
  "current_page": 1
}
```

## Security & Access Control

### Role-Based Access Control (RBAC)

1. **Parent Role**
   - Added to `UserRole` enum in `src/domain/common_details.rs`
   - Serializes as "PARENT" in uppercase

2. **Parent Guard**
   - `check_parent()` - Verifies user has Parent role
   - `check_parent_access()` - Validates parent-student relationship

3. **Access Validation Flow**
   ```
   Request → JWT Middleware → Extract User
           → Check Parent Role
           → Validate Student Ownership
           → Allow/Deny Access
   ```

### Tenant Isolation

- All parent portal endpoints require `X-School-ID` header
- Database queries filtered by `school_id`
- Parent can only access data within their school
- Student access validated through `student_ids` array

### Data Access Rules

**Parents Can:**
- View their own profile
- View linked students' attendance (read-only)
- View linked students' grades (read-only)
- View linked students' fee balance (read-only)
- View school-wide announcements
- View class-specific announcements (if child is in that class)

**Parents Cannot:**
- Modify any grades or attendance
- Access other parents' data
- Access students not linked to them
- Perform administrative operations

## Aggregation Pipelines

### Parent Pipeline
Populates parent with:
- User account
- School information
- Linked students

### Dashboard Pipeline
Aggregates:
- Total children count
- Latest announcements (5 most recent)
- Per-child summary (attendance, GPA, fees)

### Attendance Pipeline
Calculates:
- Present/Absent/Late/Excused counts
- Attendance percentage
- Recent 10 attendance records

### Results Pipeline
Retrieves:
- Latest term results
- Subject-level breakdown
- GPA and rank
- Teacher remarks

### Finance Pipeline
Aggregates:
- Total fee required
- Amount paid
- Outstanding balance
- Payment history
- Installment schedule

## Integration Points

### Reused Collections
- `students` - Student records
- `attendance` - Attendance records
- `student_term_results` - Academic results
- `scores` - Subject scores
- `announcements` - School announcements
- `classes` - Class information
- `enrollments` - Fee enrollments
- `payments` - Payment records

### Reused Services
- `AnnouncementService` - For fetching announcements
- `CloudinaryService` - For image uploads
- `EventService` - For audit logging

## Performance Optimizations

### Indexes
- All foreign keys indexed
- Compound indexes for common queries
- Partial indexes for optional fields

### Aggregation Strategy
- Single aggregation queries instead of N+1
- Pagination support at database level
- Efficient $lookup operations

### Caching
- Database handles cached in MongoManager
- Reuse of service instances where possible

## Audit Logging

Parent portal actions are logged via EventService:
- Parent login (via auth service)
- Dashboard access
- Attendance view
- Results view
- Finance view

Events broadcast with:
- Event type
- Entity ID
- School ID
- Timestamp
- User context

## Error Handling

### Consistent Error Responses
```json
{
  "message": "Error description"
}
```

### HTTP Status Codes
- `200 OK` - Successful retrieval
- `201 Created` - Successful creation
- `400 Bad Request` - Validation error
- `401 Unauthorized` - Not authenticated
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

## Testing Recommendations

### Unit Tests
- Service layer methods
- Validation functions
- Access control logic

### Integration Tests
- API endpoints
- Database operations
- Aggregation pipelines

### Security Tests
- Role-based access control
- Parent-student relationship validation
- Tenant isolation
- JWT authentication

### Test Scenarios
1. Parent can access own children's data
2. Parent cannot access other children's data
3. Admin can manage all parents
4. Staff can manage all parents
5. Dashboard aggregates correctly
6. Announcements filtered by class
7. Attendance calculations accurate
8. Results display correctly
9. Finance summary accurate

## Deployment Considerations

### Environment Variables
No new environment variables required. Uses existing:
- `MONGODB_URI`
- `JWT_SECRET`
- `CLOUDINARY_*` (for image uploads)

### Database Migration
1. Create `parents` collection
2. Apply indexes
3. Link existing guardians to parent records (if applicable)

### Rollout Strategy
1. Deploy backend with parent endpoints
2. Create parent user accounts
3. Link parents to students
4. Enable parent portal access
5. Monitor logs and performance

## Future Enhancements

### Potential Features
1. **Communication**
   - Direct messaging with teachers
   - Email notifications for grades/attendance

2. **Payments**
   - Online fee payment integration
   - Payment receipt generation

3. **Reports**
   - Downloadable progress reports
   - Attendance certificates

4. **Notifications**
   - Push notifications for important events
   - SMS alerts for absences

5. **Multi-Child Management**
   - Unified view across all children
   - Comparative analytics

6. **Calendar Integration**
   - School events calendar
   - Exam schedules
   - Parent-teacher meeting bookings

## Maintenance

### Regular Tasks
- Monitor parent access logs
- Review and optimize slow queries
- Update indexes based on usage patterns
- Clean up inactive parent accounts

### Monitoring Metrics
- API response times
- Database query performance
- Error rates
- User engagement

## Conclusion

The Parent/Guardian Portal module is production-ready, multi-tenant safe, secure, and consistent with the Space-Together backend architecture. It provides comprehensive read-only access to student data while maintaining strict security boundaries and performance optimization.
