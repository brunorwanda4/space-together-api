# Assignment & Homework System Documentation

## Overview

The Assignment & Homework System is a comprehensive module that enables teachers to create and manage assignments, students to submit their work, and teachers to grade submissions. The system includes deadline enforcement, file upload support, and role-based access control.

## Table of Contents

1. [Architecture](#architecture)
2. [Data Models](#data-models)
3. [API Endpoints](#api-endpoints)
4. [Business Logic](#business-logic)
5. [Access Control](#access-control)
6. [File Management](#file-management)
7. [Usage Examples](#usage-examples)

---

## Architecture

### Module Structure

```
src/
├── domain/
│   └── assignment.rs          # Domain models and types
├── services/
│   └── assignment_service.rs  # Business logic layer
├── api/
│   └── assignment_api.rs      # REST API endpoints
└── pipeline/
    └── assignment_pipeline.rs # MongoDB aggregation pipelines
```

### Technology Stack

- **Framework**: Actix-web
- **Database**: MongoDB
- **File Storage**: Cloudinary
- **Authentication**: JWT middleware
- **Event System**: EventService for audit logging

---

## Data Models

### Assignment

Represents a homework or assignment created by a teacher.

```rust
pub struct Assignment {
    pub id: Option<ObjectId>,
    pub school_id: Option<ObjectId>,
    pub class_id: Option<ObjectId>,
    pub subject_id: Option<ObjectId>,
    pub teacher_id: Option<ObjectId>,
    
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    
    pub due_date: DateTime<Utc>,
    pub max_score: f64,
    pub allow_late_submission: bool,
    
    pub attachment_url: Option<String>,
    pub attachment_id: Option<String>,
    
    pub status: AssignmentStatus,  // Draft, Published, Archived
    pub auto_grade_enabled: bool,
    
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Status Values:**
- `Draft`: Assignment is being prepared, not visible to students
- `Published`: Assignment is active and visible to students
- `Archived`: Assignment is completed and archived

### Submission

Represents a student's submission for an assignment.

```rust
pub struct Submission {
    pub id: Option<ObjectId>,
    pub assignment_id: Option<ObjectId>,
    pub student_id: Option<ObjectId>,
    
    pub file_url: Option<String>,
    pub file_id: Option<String>,
    pub comment: Option<String>,
    
    pub is_late: bool,
    
    pub score: Option<f64>,
    pub feedback: Option<String>,
    pub feedback_file_url: Option<String>,
    pub feedback_file_id: Option<String>,
    
    pub graded_at: Option<DateTime<Utc>>,
    pub graded_by: Option<ObjectId>,
    
    pub status: SubmissionStatus,  // Submitted, Graded
    
    pub auto_grade_score: Option<f64>,
    pub ai_feedback: Option<String>,
    
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Status Values:**
- `Submitted`: Student has submitted, awaiting grading
- `Graded`: Teacher has graded the submission

### Related Models

**AssignmentWithRelations**: Assignment with populated teacher, subject, class, and submission counts
**SubmissionWithRelations**: Submission with populated student, assignment, and grading teacher

---

## API Endpoints

### Base URL
```
/api/v1/assignments
/api/school/:school_id/assignments
```

### Assignment Management

#### 1. List All Assignments
```http
GET /assignments
```

**Query Parameters:**
- `filter` (optional): Search term
- `limit` (optional): Number of results per page
- `skip` (optional): Number of results to skip
- `field[]` (optional): Filter fields
- `value[]` (optional): Filter values

**Response:**
```json
{
  "data": [
    {
      "id": "...",
      "title": "Math Homework Chapter 5",
      "description": "Complete exercises 1-10",
      "due_date": "2026-03-01T23:59:59Z",
      "max_score": 100,
      "status": "Published",
      "teacher": { ... },
      "subject": { ... },
      "class": { ... },
      "submission_count": 15,
      "total_students": 25
    }
  ],
  "total": 50,
  "total_pages": 5,
  "current_page": 1
}
```

**Access:** Public (filtered by school)

---

#### 2. Get Assignment by ID
```http
GET /assignments/:id
```

**Response:**
```json
{
  "id": "...",
  "title": "Math Homework Chapter 5",
  "description": "Complete exercises 1-10",
  "instructions": "Show all work. Use proper notation.",
  "due_date": "2026-03-01T23:59:59Z",
  "max_score": 100,
  "allow_late_submission": true,
  "attachment_url": "https://...",
  "status": "Published",
  "teacher": { ... },
  "subject": { ... },
  "class": { ... },
  "submission_count": 15,
  "total_students": 25
}
```

**Access:** Public (filtered by school)

---

#### 3. Create Assignment
```http
POST /assignments
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "class_id": "...",
  "subject_id": "...",
  "title": "Math Homework Chapter 5",
  "description": "Complete exercises 1-10",
  "instructions": "Show all work",
  "due_date": "2026-03-01T23:59:59Z",
  "max_score": 100,
  "allow_late_submission": true,
  "attachment_url": "data:image/png;base64,...",
  "status": "Published"
}
```

**Response:**
```json
{
  "id": "...",
  "title": "Math Homework Chapter 5",
  ...
}
```

**Access:** Teachers only
**Validation:**
- Teacher must be assigned to the subject
- Title is required
- Max score must be > 0

---

#### 4. Update Assignment
```http
PUT /assignments/:id
Authorization: Bearer <token>
```

**Request Body:** (Partial update supported)
```json
{
  "title": "Updated Title",
  "due_date": "2026-03-15T23:59:59Z",
  "status": "Archived"
}
```

**Access:** Assignment creator or Admin
**Validation:**
- Only the teacher who created the assignment can update it
- Admins can update any assignment

---

#### 5. Delete Assignment
```http
DELETE /assignments/:id
Authorization: Bearer <token>
```

**Response:**
```json
{
  "id": "...",
  "title": "Math Homework Chapter 5",
  "is_deleted": true,
  "deleted_at": "2026-02-15T10:30:00Z"
}
```

**Access:** Assignment creator or Admin
**Note:** Soft delete - assignment is marked as deleted but not removed from database

---

#### 6. Count Assignments
```http
GET /assignments/count
```

**Query Parameters:** Same as list endpoint

**Response:**
```json
{
  "count": 50
}
```

---

### Student Submissions

#### 7. Submit Assignment
```http
POST /assignments/:id/submit
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "file_url": "data:application/pdf;base64,...",
  "comment": "I completed all exercises"
}
```

**Response:**
```json
{
  "id": "...",
  "assignment_id": "...",
  "student_id": "...",
  "file_url": "https://cloudinary.com/...",
  "comment": "I completed all exercises",
  "is_late": false,
  "status": "Submitted",
  "submitted_at": "2026-02-28T15:30:00Z"
}
```

**Access:** Students only
**Validation:**
- Student must be enrolled in the class
- Assignment must be published
- Deadline enforcement (unless late submissions allowed)
- Only one submission per student (use update to modify)

---

#### 8. Get Assignment Submissions
```http
GET /assignments/:id/submissions
Authorization: Bearer <token>
```

**Query Parameters:**
- `filter`, `limit`, `skip` (same as list endpoint)

**Response:**
```json
{
  "data": [
    {
      "id": "...",
      "student": {
        "id": "...",
        "name": "John Doe",
        "email": "john@example.com"
      },
      "file_url": "https://...",
      "is_late": false,
      "score": 95,
      "status": "Graded",
      "submitted_at": "2026-02-28T15:30:00Z",
      "graded_at": "2026-03-02T10:00:00Z"
    }
  ],
  "total": 15,
  "total_pages": 1,
  "current_page": 1
}
```

**Access:** Teachers, Admin, School Staff

---

#### 9. Grade Submission
```http
PUT /assignments/:assignment_id/grade/:submission_id
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "score": 95,
  "feedback": "Excellent work! Minor error in question 7.",
  "feedback_file": "data:application/pdf;base64,..."
}
```

**Response:**
```json
{
  "id": "...",
  "score": 95,
  "feedback": "Excellent work!",
  "feedback_file_url": "https://...",
  "status": "Graded",
  "graded_at": "2026-03-02T10:00:00Z",
  "graded_by": "..."
}
```

**Access:** Subject teacher or Admin
**Validation:**
- Score must not exceed max_score
- Only the assigned teacher can grade
- Audit log created for grade changes

---

#### 10. Get Submission by ID
```http
GET /submissions/:id
Authorization: Bearer <token>
```

**Response:**
```json
{
  "id": "...",
  "student": { ... },
  "assignment": { ... },
  "file_url": "https://...",
  "score": 95,
  "feedback": "Excellent work!",
  "status": "Graded",
  "graded_by_teacher": { ... }
}
```

**Access:**
- Students: Own submissions only
- Teachers/Admin: All submissions

---

#### 11. Update Submission
```http
PUT /submissions/:id
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "file_url": "data:application/pdf;base64,...",
  "comment": "Updated submission"
}
```

**Access:**
- Students: Own submissions, before grading only
- Teachers/Admin: Any submission

**Validation:**
- Students cannot update graded submissions

---

## Business Logic

### Assignment Creation Flow

1. **Validation**
   - Verify title is not empty
   - Verify max_score > 0
   - Verify teacher is assigned to the subject

2. **File Upload**
   - If attachment provided, upload to Cloudinary
   - Store secure URL and public ID

3. **Database Insert**
   - Create assignment record
   - Set timestamps

4. **Event Broadcasting**
   - Broadcast "assignment_created" event
   - Notify relevant parties

### Submission Flow

1. **Pre-Submission Validation**
   - Verify assignment exists and is published
   - Verify student is enrolled in class
   - Check for existing submission
   - Validate deadline

2. **Deadline Validation**
   ```rust
   let now = Utc::now();
   let is_late = now > assignment.due_date;
   
   if is_late && !assignment.allow_late_submission {
       return Err("Deadline passed");
   }
   ```

3. **File Upload**
   - Upload student's file to Cloudinary
   - Store secure URL and public ID

4. **Database Insert**
   - Create submission record
   - Mark as late if applicable
   - Set status to "Submitted"

5. **Event Broadcasting**
   - Broadcast "submission_created" event

### Grading Flow

1. **Validation**
   - Verify teacher is assigned to subject
   - Verify score ≤ max_score

2. **Update Submission**
   - Set score and feedback
   - Upload feedback file if provided
   - Set graded_by and graded_at
   - Change status to "Graded"

3. **Audit Logging**
   - Log grade change event
   - Include grader information

---

## Access Control

### Role-Based Permissions

| Action | Admin | Teacher | Student | Staff |
|--------|-------|---------|---------|-------|
| Create Assignment | ✅ | ✅ (own subjects) | ❌ | ❌ |
| View Assignments | ✅ | ✅ | ✅ (own class) | ✅ |
| Update Assignment | ✅ | ✅ (own only) | ❌ | ❌ |
| Delete Assignment | ✅ | ✅ (own only) | ❌ | ❌ |
| Submit Assignment | ❌ | ❌ | ✅ (own class) | ❌ |
| View Submissions | ✅ | ✅ (own subjects) | ✅ (own only) | ✅ |
| Grade Submission | ✅ | ✅ (own subjects) | ❌ | ❌ |
| Update Submission | ✅ | ✅ | ✅ (before grading) | ❌ |

### Multi-Tenant Security

All queries are automatically filtered by `school_id` from the `X-School-ID` header:

```rust
// Automatic school filtering
let db = get_database(&req, &state);  // Gets school-specific database
```

### Validation Rules

1. **Teacher-Subject Validation**
   ```rust
   // Teacher must be assigned to the subject
   if subject.teacher_id != teacher_id {
       return Err("Not assigned to this subject");
   }
   ```

2. **Student-Class Validation**
   ```rust
   // Student must be enrolled in the class
   if student.class_id != assignment.class_id {
       return Err("Not enrolled in this class");
   }
   ```

3. **Ownership Validation**
   ```rust
   // Only assignment creator can update/delete
   if assignment.teacher_id != current_user_teacher_id && !is_admin {
       return Err("Not authorized");
   }
   ```

---

## File Management

### Supported File Types

**Assignments:**
- PDF documents
- Word documents (DOC, DOCX)
- Images (PNG, JPG, GIF)
- Text files

**Submissions:**
- Same as assignments

### File Upload Process

1. **Client Side**
   - Convert file to base64 data URI
   - Include in request body

2. **Server Side**
   ```rust
   // Upload to Cloudinary
   let cloud_res = CloudinaryService::upload_to_cloudinary(&file_data).await?;
   
   // Store URLs
   assignment.attachment_url = Some(cloud_res.secure_url);
   assignment.attachment_id = Some(cloud_res.public_id);
   ```

3. **File Deletion**
   ```rust
   // When updating/deleting
   if let Some(old_file_id) = old_attachment_id {
       CloudinaryService::delete_from_cloudinary(&old_file_id).await.ok();
   }
   ```

### File Size Limits

- Maximum file size: 5MB
- Enforced at Cloudinary service level

---

## Usage Examples

### Example 1: Teacher Creates Assignment

```javascript
// POST /api/v1/assignments
const response = await fetch('/api/v1/assignments', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer <token>',
    'Content-Type': 'application/json',
    'X-School-ID': 'school123'
  },
  body: JSON.stringify({
    class_id: '507f1f77bcf86cd799439011',
    subject_id: '507f1f77bcf86cd799439012',
    title: 'Chapter 5 Homework',
    description: 'Complete all exercises',
    instructions: 'Show your work for full credit',
    due_date: '2026-03-01T23:59:59Z',
    max_score: 100,
    allow_late_submission: true,
    status: 'Published'
  })
});

const assignment = await response.json();
console.log('Created:', assignment.id);
```

### Example 2: Student Submits Assignment

```javascript
// Convert file to base64
const fileInput = document.getElementById('file');
const file = fileInput.files[0];
const reader = new FileReader();

reader.onload = async (e) => {
  const base64 = e.target.result;
  
  // POST /api/v1/assignments/:id/submit
  const response = await fetch(`/api/v1/assignments/${assignmentId}/submit`, {
    method: 'POST',
    headers: {
      'Authorization': 'Bearer <token>',
      'Content-Type': 'application/json',
      'X-School-ID': 'school123'
    },
    body: JSON.stringify({
      file_url: base64,
      comment: 'Completed all exercises'
    })
  });
  
  const submission = await response.json();
  console.log('Submitted:', submission.id);
};

reader.readAsDataURL(file);
```

### Example 3: Teacher Grades Submission

```javascript
// PUT /api/v1/assignments/:assignment_id/grade/:submission_id
const response = await fetch(
  `/api/v1/assignments/${assignmentId}/grade/${submissionId}`,
  {
    method: 'PUT',
    headers: {
      'Authorization': 'Bearer <token>',
      'Content-Type': 'application/json',
      'X-School-ID': 'school123'
    },
    body: JSON.stringify({
      score: 95,
      feedback: 'Excellent work! Minor error in question 7.'
    })
  }
);

const graded = await response.json();
console.log('Graded:', graded.score);
```

### Example 4: List Assignments with Filters

```javascript
// GET /api/v1/assignments?field[]=class_id&value[]=...&limit=10
const params = new URLSearchParams({
  'field[]': 'class_id',
  'value[]': '507f1f77bcf86cd799439011',
  'field[]': 'status',
  'value[]': 'Published',
  limit: 10,
  skip: 0
});

const response = await fetch(`/api/v1/assignments?${params}`, {
  headers: {
    'X-School-ID': 'school123'
  }
});

const result = await response.json();
console.log(`Found ${result.total} assignments`);
result.data.forEach(assignment => {
  console.log(`- ${assignment.title} (${assignment.submission_count}/${assignment.total_students} submitted)`);
});
```

### Example 5: View Submission Statistics

```javascript
// GET /api/v1/assignments/:id/submissions
const response = await fetch(`/api/v1/assignments/${assignmentId}/submissions`, {
  headers: {
    'Authorization': 'Bearer <token>',
    'X-School-ID': 'school123'
  }
});

const result = await response.json();

const stats = {
  total: result.total,
  graded: result.data.filter(s => s.status === 'Graded').length,
  late: result.data.filter(s => s.is_late).length,
  avgScore: result.data
    .filter(s => s.score)
    .reduce((sum, s) => sum + s.score, 0) / result.data.filter(s => s.score).length
};

console.log('Submission Statistics:', stats);
```

---

## Database Indexes

### Assignments Collection

```javascript
{
  school_id: 1,
  class_id: 1,
  subject_id: 1,
  teacher_id: 1,
  due_date: 1,
  status: 1,
  is_deleted: 1,
  { school_id: 1, class_id: 1 },
  { school_id: 1, subject_id: 1 },
  { school_id: 1, teacher_id: 1 }
}
```

### Submissions Collection

```javascript
{
  assignment_id: 1,
  student_id: 1,
  graded_by: 1,
  submitted_at: 1,
  status: 1,
  is_deleted: 1,
  { assignment_id: 1, student_id: 1 } // Unique
}
```

---

## Future Enhancements

### AI Auto-Grading (Placeholder)

The system includes placeholder fields for future AI integration:

```rust
pub struct Submission {
    // ...
    pub auto_grade_enabled: bool,
    pub auto_grade_score: Option<f64>,
    pub ai_feedback: Option<String>,
}

// Placeholder method
pub async fn trigger_auto_grading(&self, submission_id: &IdType) -> Result<(), AppError> {
    // Future: Send to AI service for grading
    // Update submission with auto_grade_score and ai_feedback
    Ok(())
}
```

### Planned Features

1. **Rubric-Based Grading**
   - Define grading criteria
   - Score per criterion
   - Weighted scoring

2. **Peer Review**
   - Students review each other's work
   - Anonymous feedback
   - Teacher moderation

3. **Plagiarism Detection**
   - Integration with plagiarism detection services
   - Similarity reports

4. **Analytics Dashboard**
   - Submission trends
   - Grade distribution
   - Student performance tracking

5. **Notifications**
   - Email/push notifications for:
     - New assignments
     - Upcoming deadlines
     - Graded submissions

---

## Error Handling

### Common Error Responses

```json
{
  "message": "Assignment not found"
}
```

```json
{
  "message": "Submission deadline has passed and late submissions are not allowed"
}
```

```json
{
  "message": "You are not enrolled in this class"
}
```

```json
{
  "message": "Score cannot exceed max score of 100"
}
```

### HTTP Status Codes

- `200 OK`: Successful GET/PUT
- `201 Created`: Successful POST
- `400 Bad Request`: Validation error
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

---

## Testing

### Manual Testing Checklist

**Assignment Creation:**
- [ ] Teacher can create assignment
- [ ] Non-teacher cannot create assignment
- [ ] Teacher must be assigned to subject
- [ ] Title is required
- [ ] Max score must be > 0
- [ ] File upload works

**Student Submission:**
- [ ] Student can submit before deadline
- [ ] Student cannot submit after deadline (if not allowed)
- [ ] Student can submit late (if allowed)
- [ ] Student must be enrolled in class
- [ ] Only one submission per student
- [ ] File upload works

**Grading:**
- [ ] Teacher can grade own subject submissions
- [ ] Teacher cannot grade other subject submissions
- [ ] Score validation works
- [ ] Feedback file upload works
- [ ] Audit log created

**Access Control:**
- [ ] Students see only their class assignments
- [ ] Teachers see only their subject assignments
- [ ] Admin sees all assignments
- [ ] Multi-tenant isolation works

---

## Troubleshooting

### Issue: "Teacher is not assigned to this subject"

**Cause:** The teacher creating the assignment is not assigned to the specified subject.

**Solution:** Verify the teacher is assigned to the subject in the `class_subjects` collection.

### Issue: "You have already submitted this assignment"

**Cause:** Student is trying to create a new submission when one already exists.

**Solution:** Use the update submission endpoint instead.

### Issue: "Submission deadline has passed"

**Cause:** Student is trying to submit after the due date and late submissions are not allowed.

**Solution:** 
- Check `allow_late_submission` flag
- Contact teacher to extend deadline or enable late submissions

### Issue: "Score cannot exceed max score"

**Cause:** Teacher is trying to assign a score higher than the assignment's max_score.

**Solution:** Verify the max_score value and adjust the score accordingly.

---

## Support

For issues or questions:
1. Check this documentation
2. Review the API endpoint examples
3. Check server logs for detailed error messages
4. Contact the development team

---

## Changelog

### Version 1.0.0 (2026-02-15)
- Initial implementation
- Assignment CRUD operations
- Student submission system
- Teacher grading functionality
- File upload support
- Role-based access control
- Multi-tenant support
- Audit logging
- Soft delete support
- AI auto-grading placeholders
