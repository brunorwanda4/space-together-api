# Assignment & Homework API Reference

## Authentication

All protected endpoints require a JWT token in the Authorization header:

```http
Authorization: Bearer <your_jwt_token>
```

## Headers

### Required Headers

```http
Content-Type: application/json
X-School-ID: <school_id>
```

---

## Assignments

### List Assignments

Retrieve a paginated list of assignments with related data.

```http
GET /assignments
```

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| filter | string | No | Search term for filtering |
| limit | integer | No | Number of results per page (default: 10) |
| skip | integer | No | Number of results to skip (default: 0) |
| field[] | string[] | No | Filter field names |
| value[] | string[] | No | Filter field values |

#### Example Request

```bash
curl -X GET "https://api.space-together.com/api/v1/assignments?limit=10&skip=0" \
  -H "X-School-ID: 507f1f77bcf86cd799439011"
```

#### Example Response

```json
{
  "data": [
    {
      "id": "507f1f77bcf86cd799439013",
      "school_id": "507f1f77bcf86cd799439011",
      "class_id": "507f1f77bcf86cd799439014",
      "subject_id": "507f1f77bcf86cd799439015",
      "teacher_id": "507f1f77bcf86cd799439016",
      "title": "Chapter 5 Homework",
      "description": "Complete all exercises from chapter 5",
      "instructions": "Show your work for full credit",
      "due_date": "2026-03-01T23:59:59Z",
      "max_score": 100,
      "allow_late_submission": true,
      "attachment_url": "https://res.cloudinary.com/...",
      "attachment_id": "assignment_1234567890",
      "status": "Published",
      "auto_grade_enabled": false,
      "is_deleted": false,
      "created_at": "2026-02-15T10:00:00Z",
      "updated_at": "2026-02-15T10:00:00Z",
      "teacher": {
        "id": "507f1f77bcf86cd799439016",
        "name": "Jane Smith",
        "email": "jane.smith@school.com"
      },
      "subject": {
        "id": "507f1f77bcf86cd799439015",
        "name": "Mathematics",
        "code": "MATH101"
      },
      "class": {
        "id": "507f1f77bcf86cd799439014",
        "name": "Grade 10A"
      },
      "submission_count": 15,
      "total_students": 25
    }
  ],
  "total": 50,
  "total_pages": 5,
  "current_page": 1
}
```

#### Status Codes

- `200 OK`: Success
- `400 Bad Request`: Invalid parameters
- `500 Internal Server Error`: Server error

---

### Get Assignment by ID

Retrieve a single assignment with full details.

```http
GET /assignments/:id
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Assignment ID |

#### Example Request

```bash
curl -X GET "https://api.space-together.com/api/v1/assignments/507f1f77bcf86cd799439013" \
  -H "X-School-ID: 507f1f77bcf86cd799439011"
```

#### Example Response

```json
{
  "id": "507f1f77bcf86cd799439013",
  "title": "Chapter 5 Homework",
  "description": "Complete all exercises from chapter 5",
  "instructions": "Show your work for full credit",
  "due_date": "2026-03-01T23:59:59Z",
  "max_score": 100,
  "allow_late_submission": true,
  "attachment_url": "https://res.cloudinary.com/...",
  "status": "Published",
  "teacher": { ... },
  "subject": { ... },
  "class": { ... },
  "submission_count": 15,
  "total_students": 25
}
```

#### Status Codes

- `200 OK`: Success
- `404 Not Found`: Assignment not found
- `500 Internal Server Error`: Server error

---

### Create Assignment

Create a new assignment. Requires teacher role.

```http
POST /assignments
Authorization: Bearer <token>
```

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| class_id | string | Yes | Class ID |
| subject_id | string | Yes | Subject ID |
| title | string | Yes | Assignment title |
| description | string | No | Assignment description |
| instructions | string | No | Detailed instructions |
| due_date | string (ISO 8601) | Yes | Deadline |
| max_score | number | Yes | Maximum score (> 0) |
| allow_late_submission | boolean | No | Allow late submissions (default: false) |
| attachment_url | string | No | Base64 data URI or URL |
| status | string | No | Draft, Published, Archived (default: Published) |

#### Example Request

```bash
curl -X POST "https://api.space-together.com/api/v1/assignments" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: 507f1f77bcf86cd799439011" \
  -d '{
    "class_id": "507f1f77bcf86cd799439014",
    "subject_id": "507f1f77bcf86cd799439015",
    "title": "Chapter 5 Homework",
    "description": "Complete all exercises",
    "instructions": "Show your work",
    "due_date": "2026-03-01T23:59:59Z",
    "max_score": 100,
    "allow_late_submission": true,
    "status": "Published"
  }'
```

#### Example Response

```json
{
  "id": "507f1f77bcf86cd799439013",
  "class_id": "507f1f77bcf86cd799439014",
  "subject_id": "507f1f77bcf86cd799439015",
  "teacher_id": "507f1f77bcf86cd799439016",
  "title": "Chapter 5 Homework",
  "description": "Complete all exercises",
  "due_date": "2026-03-01T23:59:59Z",
  "max_score": 100,
  "status": "Published",
  "created_at": "2026-02-15T10:00:00Z",
  "updated_at": "2026-02-15T10:00:00Z"
}
```

#### Status Codes

- `201 Created`: Success
- `400 Bad Request`: Validation error
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not a teacher or not assigned to subject
- `500 Internal Server Error`: Server error

#### Validation Rules

- Title cannot be empty
- Max score must be greater than 0
- Teacher must be assigned to the specified subject
- Due date must be a valid ISO 8601 datetime

---

### Update Assignment

Update an existing assignment. Only the creator or admin can update.

```http
PUT /assignments/:id
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Assignment ID |

#### Request Body

All fields are optional (partial update supported).

```json
{
  "title": "Updated Title",
  "description": "Updated description",
  "due_date": "2026-03-15T23:59:59Z",
  "max_score": 120,
  "allow_late_submission": false,
  "status": "Archived"
}
```

#### Example Request

```bash
curl -X PUT "https://api.space-together.com/api/v1/assignments/507f1f77bcf86cd799439013" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: 507f1f77bcf86cd799439011" \
  -d '{
    "title": "Updated Chapter 5 Homework",
    "due_date": "2026-03-15T23:59:59Z"
  }'
```

#### Status Codes

- `200 OK`: Success
- `400 Bad Request`: Validation error
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not the creator or admin
- `404 Not Found`: Assignment not found
- `500 Internal Server Error`: Server error

---

### Delete Assignment

Soft delete an assignment. Only the creator or admin can delete.

```http
DELETE /assignments/:id
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Assignment ID |

#### Example Request

```bash
curl -X DELETE "https://api.space-together.com/api/v1/assignments/507f1f77bcf86cd799439013" \
  -H "Authorization: Bearer <token>" \
  -H "X-School-ID: 507f1f77bcf86cd799439011"
```

#### Example Response

```json
{
  "id": "507f1f77bcf86cd799439013",
  "title": "Chapter 5 Homework",
  "is_deleted": true,
  "deleted_at": "2026-02-15T15:30:00Z"
}
```

#### Status Codes

- `200 OK`: Success
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not the creator or admin
- `404 Not Found`: Assignment not found
- `500 Internal Server Error`: Server error

---

### Count Assignments

Get the total count of assignments matching the filter criteria.

```http
GET /assignments/count
```

#### Query Parameters

Same as List Assignments endpoint.

#### Example Request

```bash
curl -X GET "https://api.space-together.com/api/v1/assignments/count?field[]=status&value[]=Published" \
  -H "X-School-ID: 507f1f77bcf86cd799439011"
```

#### Example Response

```json
{
  "count": 42
}
```

#### Status Codes

- `200 OK`: Success
- `400 Bad Request`: Invalid parameters
- `500 Internal Server Error`: Server error

---

## Submissions

### Submit Assignment

Submit an assignment as a student.

```http
POST /assignments/:id/submit
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Assignment ID |

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| file_url | string | No | Base64 data URI or URL |
| comment | string | No | Student comment |

#### Example Request

```bash
curl -X POST "https://api.space-together.com/api/v1/assignments/507f1f77bcf86cd799439013/submit" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: 507f1f77bcf86cd799439011" \
  -d '{
    "file_url": "data:application/pdf;base64,JVBERi0xLjQKJ...",
    "comment": "I completed all exercises"
  }'
```

#### Example Response

```json
{
  "id": "507f1f77bcf86cd799439020",
  "assignment_id": "507f1f77bcf86cd799439013",
  "student_id": "507f1f77bcf86cd799439021",
  "file_url": "https://res.cloudinary.com/...",
  "file_id": "submission_1234567890",
  "comment": "I completed all exercises",
  "is_late": false,
  "status": "Submitted",
  "submitted_at": "2026-02-28T15:30:00Z",
  "updated_at": "2026-02-28T15:30:00Z"
}
```

#### Status Codes

- `201 Created`: Success
- `400 Bad Request`: Validation error or already submitted
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not a student or not enrolled in class
- `404 Not Found`: Assignment not found
- `500 Internal Server Error`: Server error

#### Validation Rules

- Student must be enrolled in the assignment's class
- Assignment must be published
- Deadline enforcement (unless late submissions allowed)
- Only one submission per student (use update to modify)

---

### Get Assignment Submissions

Get all submissions for an assignment. Teachers and admins only.

```http
GET /assignments/:id/submissions
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Assignment ID |

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| filter | string | No | Search term |
| limit | integer | No | Results per page |
| skip | integer | No | Results to skip |

#### Example Request

```bash
curl -X GET "https://api.space-together.com/api/v1/assignments/507f1f77bcf86cd799439013/submissions" \
  -H "Authorization: Bearer <token>" \
  -H "X-School-ID: 507f1f77bcf86cd799439011"
```

#### Example Response

```json
{
  "data": [
    {
      "id": "507f1f77bcf86cd799439020",
      "assignment_id": "507f1f77bcf86cd799439013",
      "student_id": "507f1f77bcf86cd799439021",
      "file_url": "https://res.cloudinary.com/...",
      "comment": "I completed all exercises",
      "is_late": false,
      "score": 95,
      "feedback": "Excellent work!",
      "status": "Graded",
      "submitted_at": "2026-02-28T15:30:00Z",
      "graded_at": "2026-03-02T10:00:00Z",
      "student": {
        "id": "507f1f77bcf86cd799439021",
        "name": "John Doe",
        "email": "john.doe@school.com"
      },
      "graded_by_teacher": {
        "id": "507f1f77bcf86cd799439016",
        "name": "Jane Smith"
      }
    }
  ],
  "total": 15,
  "total_pages": 1,
  "current_page": 1
}
```

#### Status Codes

- `200 OK`: Success
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not a teacher, admin, or staff
- `404 Not Found`: Assignment not found
- `500 Internal Server Error`: Server error

---

### Grade Submission

Grade a student's submission. Teachers and admins only.

```http
PUT /assignments/:assignment_id/grade/:submission_id
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| assignment_id | string | Yes | Assignment ID |
| submission_id | string | Yes | Submission ID |

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| score | number | Yes | Score (≤ max_score) |
| feedback | string | No | Teacher feedback |
| feedback_file | string | No | Base64 data URI or URL |

#### Example Request

```bash
curl -X PUT "https://api.space-together.com/api/v1/assignments/507f1f77bcf86cd799439013/grade/507f1f77bcf86cd799439020" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: 507f1f77bcf86cd799439011" \
  -d '{
    "score": 95,
    "feedback": "Excellent work! Minor error in question 7."
  }'
```

#### Example Response

```json
{
  "id": "507f1f77bcf86cd799439020",
  "assignment_id": "507f1f77bcf86cd799439013",
  "student_id": "507f1f77bcf86cd799439021",
  "score": 95,
  "feedback": "Excellent work! Minor error in question 7.",
  "status": "Graded",
  "graded_at": "2026-03-02T10:00:00Z",
  "graded_by": "507f1f77bcf86cd799439016",
  "updated_at": "2026-03-02T10:00:00Z"
}
```

#### Status Codes

- `200 OK`: Success
- `400 Bad Request`: Validation error (score > max_score)
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not the subject teacher or admin
- `404 Not Found`: Assignment or submission not found
- `500 Internal Server Error`: Server error

#### Validation Rules

- Score must not exceed assignment's max_score
- Only the assigned teacher or admin can grade
- Audit log is created for grade changes

---

### Get Submission by ID

Get a single submission with full details.

```http
GET /submissions/:id
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Submission ID |

#### Example Request

```bash
curl -X GET "https://api.space-together.com/api/v1/submissions/507f1f77bcf86cd799439020" \
  -H "Authorization: Bearer <token>" \
  -H "X-School-ID: 507f1f77bcf86cd799439011"
```

#### Example Response

```json
{
  "id": "507f1f77bcf86cd799439020",
  "assignment_id": "507f1f77bcf86cd799439013",
  "student_id": "507f1f77bcf86cd799439021",
  "file_url": "https://res.cloudinary.com/...",
  "comment": "I completed all exercises",
  "is_late": false,
  "score": 95,
  "feedback": "Excellent work!",
  "feedback_file_url": "https://res.cloudinary.com/...",
  "status": "Graded",
  "submitted_at": "2026-02-28T15:30:00Z",
  "graded_at": "2026-03-02T10:00:00Z",
  "student": {
    "id": "507f1f77bcf86cd799439021",
    "name": "John Doe",
    "email": "john.doe@school.com"
  },
  "assignment": {
    "id": "507f1f77bcf86cd799439013",
    "title": "Chapter 5 Homework",
    "max_score": 100
  },
  "graded_by_teacher": {
    "id": "507f1f77bcf86cd799439016",
    "name": "Jane Smith"
  }
}
```

#### Status Codes

- `200 OK`: Success
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized (students can only view own submissions)
- `404 Not Found`: Submission not found
- `500 Internal Server Error`: Server error

---

### Update Submission

Update a submission. Students can update before grading, teachers/admins anytime.

```http
PUT /submissions/:id
Authorization: Bearer <token>
```

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | string | Yes | Submission ID |

#### Request Body

All fields are optional.

```json
{
  "file_url": "data:application/pdf;base64,...",
  "comment": "Updated submission with corrections"
}
```

#### Example Request

```bash
curl -X PUT "https://api.space-together.com/api/v1/submissions/507f1f77bcf86cd799439020" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: 507f1f77bcf86cd799439011" \
  -d '{
    "comment": "Updated with corrections"
  }'
```

#### Status Codes

- `200 OK`: Success
- `400 Bad Request`: Validation error
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized or submission already graded
- `404 Not Found`: Submission not found
- `500 Internal Server Error`: Server error

#### Validation Rules

- Students cannot update graded submissions
- Students can only update their own submissions
- Teachers/admins can update any submission

---

## Error Responses

All error responses follow this format:

```json
{
  "message": "Error description"
}
```

### Common Error Messages

| Message | Cause | Solution |
|---------|-------|----------|
| "Assignment not found" | Invalid assignment ID | Verify the ID exists |
| "Submission deadline has passed and late submissions are not allowed" | Submitting after deadline | Check allow_late_submission flag |
| "You have already submitted this assignment. Use update instead." | Duplicate submission | Use PUT /submissions/:id |
| "Teacher is not assigned to this subject" | Teacher not assigned | Verify teacher-subject assignment |
| "You are not enrolled in this class" | Student not in class | Verify student enrollment |
| "Score cannot exceed max score of X" | Score too high | Adjust score or max_score |
| "Only teachers can create assignments" | Wrong role | Use teacher account |
| "You can only update your own assignments" | Not owner | Use assignment creator account |
| "Cannot update a graded submission" | Submission graded | Contact teacher |

---

## Rate Limiting

- Rate limit: 100 requests per minute per IP
- Rate limit headers included in response:
  ```
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 95
  X-RateLimit-Reset: 1645876800
  ```

---

## Pagination

All list endpoints support pagination:

```
GET /assignments?limit=10&skip=20
```

Response includes pagination metadata:

```json
{
  "data": [...],
  "total": 100,
  "total_pages": 10,
  "current_page": 3
}
```

---

## Filtering

Use `field[]` and `value[]` arrays for filtering:

```
GET /assignments?field[]=status&value[]=Published&field[]=class_id&value[]=507f...
```

Supported operators:
- Exact match (default)
- ObjectId conversion (automatic for *_id fields)

---

## Webhooks

Event notifications are sent via the EventService:

- `assignment_created`
- `assignment_updated`
- `assignment_deleted`
- `submission_created`
- `submission_updated`
- `submission_graded`

Configure webhooks in your school settings.

---

## SDK Examples

### JavaScript/TypeScript

```typescript
import { AssignmentAPI } from '@space-together/sdk';

const api = new AssignmentAPI({
  baseURL: 'https://api.space-together.com/api/v1',
  token: 'your_jwt_token',
  schoolId: 'your_school_id'
});

// Create assignment
const assignment = await api.createAssignment({
  class_id: '...',
  subject_id: '...',
  title: 'Chapter 5 Homework',
  due_date: '2026-03-01T23:59:59Z',
  max_score: 100
});

// Submit assignment
const submission = await api.submitAssignment(assignment.id, {
  file_url: fileDataURI,
  comment: 'Completed'
});

// Grade submission
const graded = await api.gradeSubmission(
  assignment.id,
  submission.id,
  {
    score: 95,
    feedback: 'Great work!'
  }
);
```

### Python

```python
from space_together import AssignmentAPI

api = AssignmentAPI(
    base_url='https://api.space-together.com/api/v1',
    token='your_jwt_token',
    school_id='your_school_id'
)

# Create assignment
assignment = api.create_assignment(
    class_id='...',
    subject_id='...',
    title='Chapter 5 Homework',
    due_date='2026-03-01T23:59:59Z',
    max_score=100
)

# Submit assignment
submission = api.submit_assignment(
    assignment['id'],
    file_url=file_data_uri,
    comment='Completed'
)

# Grade submission
graded = api.grade_submission(
    assignment['id'],
    submission['id'],
    score=95,
    feedback='Great work!'
)
```

---

## Postman Collection

Import our Postman collection for easy testing:

[Download Postman Collection](https://api.space-together.com/postman/assignments.json)

---

## Support

- Documentation: https://docs.space-together.com
- API Status: https://status.space-together.com
- Support Email: api-support@space-together.com
- Developer Forum: https://forum.space-together.com

---

## Changelog

### v1.0.0 (2026-02-15)
- Initial release
- Assignment CRUD operations
- Submission system
- Grading functionality
- File upload support
