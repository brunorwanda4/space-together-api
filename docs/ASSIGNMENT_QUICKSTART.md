# Assignment System Quick Start Guide

## Introduction

This guide will help you get started with the Assignment & Homework System in 5 minutes.

## Prerequisites

- Active Space-Together account
- JWT authentication token
- School ID
- Appropriate role (Teacher for creating, Student for submitting)

---

## Quick Start: Teacher Workflow

### Step 1: Create an Assignment

```bash
curl -X POST "https://api.space-together.com/api/v1/assignments" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: YOUR_SCHOOL_ID" \
  -d '{
    "class_id": "CLASS_ID",
    "subject_id": "SUBJECT_ID",
    "title": "Math Homework - Chapter 5",
    "description": "Complete exercises 1-10",
    "instructions": "Show all work for full credit",
    "due_date": "2026-03-01T23:59:59Z",
    "max_score": 100,
    "allow_late_submission": true,
    "status": "Published"
  }'
```

**Response:**
```json
{
  "id": "ASSIGNMENT_ID",
  "title": "Math Homework - Chapter 5",
  "status": "Published",
  "created_at": "2026-02-15T10:00:00Z"
}
```

### Step 2: View Submissions

```bash
curl -X GET "https://api.space-together.com/api/v1/assignments/ASSIGNMENT_ID/submissions" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-School-ID: YOUR_SCHOOL_ID"
```

### Step 3: Grade a Submission

```bash
curl -X PUT "https://api.space-together.com/api/v1/assignments/ASSIGNMENT_ID/grade/SUBMISSION_ID" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: YOUR_SCHOOL_ID" \
  -d '{
    "score": 95,
    "feedback": "Excellent work! Minor error in question 7."
  }'
```

---

## Quick Start: Student Workflow

### Step 1: View Available Assignments

```bash
curl -X GET "https://api.space-together.com/api/v1/assignments?field[]=class_id&value[]=YOUR_CLASS_ID" \
  -H "X-School-ID: YOUR_SCHOOL_ID"
```

### Step 2: Submit Assignment

First, convert your file to base64:

```javascript
// In browser
const fileInput = document.getElementById('file');
const file = fileInput.files[0];
const reader = new FileReader();

reader.onload = function(e) {
  const base64 = e.target.result;
  // Use base64 in API call
};

reader.readAsDataURL(file);
```

Then submit:

```bash
curl -X POST "https://api.space-together.com/api/v1/assignments/ASSIGNMENT_ID/submit" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: YOUR_SCHOOL_ID" \
  -d '{
    "file_url": "data:application/pdf;base64,JVBERi0xLjQK...",
    "comment": "I completed all exercises"
  }'
```

### Step 3: Check Your Grade

```bash
curl -X GET "https://api.space-together.com/api/v1/submissions/SUBMISSION_ID" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-School-ID: YOUR_SCHOOL_ID"
```

---

## Common Use Cases

### 1. Create Assignment with Attachment

```javascript
// JavaScript example
const createAssignmentWithFile = async (file) => {
  // Convert file to base64
  const base64 = await fileToBase64(file);
  
  const response = await fetch('/api/v1/assignments', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
      'X-School-ID': schoolId
    },
    body: JSON.stringify({
      class_id: classId,
      subject_id: subjectId,
      title: 'Homework with Instructions',
      due_date: '2026-03-01T23:59:59Z',
      max_score: 100,
      attachment_url: base64,  // File will be uploaded to Cloudinary
      status: 'Published'
    })
  });
  
  return await response.json();
};

function fileToBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
```

### 2. Filter Assignments by Status

```bash
# Get only published assignments
curl -X GET "https://api.space-together.com/api/v1/assignments?field[]=status&value[]=Published" \
  -H "X-School-ID: YOUR_SCHOOL_ID"
```

### 3. Get Overdue Assignments

```javascript
const getOverdueAssignments = async () => {
  const response = await fetch('/api/v1/assignments', {
    headers: {
      'X-School-ID': schoolId
    }
  });
  
  const data = await response.json();
  const now = new Date();
  
  return data.data.filter(assignment => {
    const dueDate = new Date(assignment.due_date);
    return dueDate < now && assignment.status === 'Published';
  });
};
```

### 4. Bulk Grade Submissions

```javascript
const bulkGrade = async (assignmentId, grades) => {
  const promises = grades.map(({ submissionId, score, feedback }) =>
    fetch(`/api/v1/assignments/${assignmentId}/grade/${submissionId}`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        'X-School-ID': schoolId
      },
      body: JSON.stringify({ score, feedback })
    })
  );
  
  return await Promise.all(promises);
};

// Usage
await bulkGrade('ASSIGNMENT_ID', [
  { submissionId: 'SUB1', score: 95, feedback: 'Great!' },
  { submissionId: 'SUB2', score: 88, feedback: 'Good work' },
  { submissionId: 'SUB3', score: 92, feedback: 'Excellent' }
]);
```

### 5. Calculate Class Average

```javascript
const getClassAverage = async (assignmentId) => {
  const response = await fetch(
    `/api/v1/assignments/${assignmentId}/submissions`,
    {
      headers: {
        'Authorization': `Bearer ${token}`,
        'X-School-ID': schoolId
      }
    }
  );
  
  const data = await response.json();
  const gradedSubmissions = data.data.filter(s => s.score !== null);
  
  if (gradedSubmissions.length === 0) return 0;
  
  const total = gradedSubmissions.reduce((sum, s) => sum + s.score, 0);
  return total / gradedSubmissions.length;
};
```

---

## Frontend Integration Examples

### React Component: Assignment List

```jsx
import React, { useEffect, useState } from 'react';

function AssignmentList({ classId }) {
  const [assignments, setAssignments] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchAssignments();
  }, [classId]);

  const fetchAssignments = async () => {
    try {
      const response = await fetch(
        `/api/v1/assignments?field[]=class_id&value[]=${classId}`,
        {
          headers: {
            'X-School-ID': localStorage.getItem('schoolId')
          }
        }
      );
      const data = await response.json();
      setAssignments(data.data);
    } catch (error) {
      console.error('Error fetching assignments:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div className="assignment-list">
      <h2>Assignments</h2>
      {assignments.map(assignment => (
        <div key={assignment.id} className="assignment-card">
          <h3>{assignment.title}</h3>
          <p>{assignment.description}</p>
          <p>Due: {new Date(assignment.due_date).toLocaleDateString()}</p>
          <p>
            Submissions: {assignment.submission_count} / {assignment.total_students}
          </p>
          <span className={`status ${assignment.status.toLowerCase()}`}>
            {assignment.status}
          </span>
        </div>
      ))}
    </div>
  );
}

export default AssignmentList;
```

### React Component: Submit Assignment

```jsx
import React, { useState } from 'react';

function SubmitAssignment({ assignmentId, onSuccess }) {
  const [file, setFile] = useState(null);
  const [comment, setComment] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const handleFileChange = (e) => {
    setFile(e.target.files[0]);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setSubmitting(true);

    try {
      // Convert file to base64
      const base64 = await fileToBase64(file);

      const response = await fetch(
        `/api/v1/assignments/${assignmentId}/submit`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json',
            'X-School-ID': localStorage.getItem('schoolId')
          },
          body: JSON.stringify({
            file_url: base64,
            comment
          })
        }
      );

      if (response.ok) {
        const data = await response.json();
        onSuccess(data);
        alert('Assignment submitted successfully!');
      } else {
        const error = await response.json();
        alert(`Error: ${error.message}`);
      }
    } catch (error) {
      console.error('Submission error:', error);
      alert('Failed to submit assignment');
    } finally {
      setSubmitting(false);
    }
  };

  const fileToBase64 = (file) => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  };

  return (
    <form onSubmit={handleSubmit} className="submit-form">
      <h3>Submit Assignment</h3>
      
      <div className="form-group">
        <label>Upload File:</label>
        <input
          type="file"
          onChange={handleFileChange}
          accept=".pdf,.doc,.docx,.jpg,.png"
          required
        />
      </div>

      <div className="form-group">
        <label>Comment (optional):</label>
        <textarea
          value={comment}
          onChange={(e) => setComment(e.target.value)}
          rows="4"
          placeholder="Add any comments about your submission..."
        />
      </div>

      <button type="submit" disabled={!file || submitting}>
        {submitting ? 'Submitting...' : 'Submit Assignment'}
      </button>
    </form>
  );
}

export default SubmitAssignment;
```

### Vue Component: Grade Submission

```vue
<template>
  <div class="grade-submission">
    <h3>Grade Submission</h3>
    
    <div class="student-info">
      <p><strong>Student:</strong> {{ submission.student.name }}</p>
      <p><strong>Submitted:</strong> {{ formatDate(submission.submitted_at) }}</p>
      <p v-if="submission.is_late" class="late-badge">Late Submission</p>
    </div>

    <div class="submission-file">
      <a :href="submission.file_url" target="_blank">View Submission</a>
    </div>

    <form @submit.prevent="submitGrade">
      <div class="form-group">
        <label>Score (max {{ maxScore }}):</label>
        <input
          v-model.number="score"
          type="number"
          :max="maxScore"
          min="0"
          step="0.5"
          required
        />
      </div>

      <div class="form-group">
        <label>Feedback:</label>
        <textarea
          v-model="feedback"
          rows="4"
          placeholder="Provide feedback to the student..."
        />
      </div>

      <button type="submit" :disabled="grading">
        {{ grading ? 'Grading...' : 'Submit Grade' }}
      </button>
    </form>
  </div>
</template>

<script>
export default {
  name: 'GradeSubmission',
  props: {
    submission: Object,
    assignmentId: String,
    maxScore: Number
  },
  data() {
    return {
      score: this.submission.score || 0,
      feedback: this.submission.feedback || '',
      grading: false
    };
  },
  methods: {
    async submitGrade() {
      if (this.score > this.maxScore) {
        alert(`Score cannot exceed ${this.maxScore}`);
        return;
      }

      this.grading = true;

      try {
        const response = await fetch(
          `/api/v1/assignments/${this.assignmentId}/grade/${this.submission.id}`,
          {
            method: 'PUT',
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('token')}`,
              'Content-Type': 'application/json',
              'X-School-ID': localStorage.getItem('schoolId')
            },
            body: JSON.stringify({
              score: this.score,
              feedback: this.feedback
            })
          }
        );

        if (response.ok) {
          const data = await response.json();
          this.$emit('graded', data);
          alert('Grade submitted successfully!');
        } else {
          const error = await response.json();
          alert(`Error: ${error.message}`);
        }
      } catch (error) {
        console.error('Grading error:', error);
        alert('Failed to submit grade');
      } finally {
        this.grading = false;
      }
    },
    formatDate(dateString) {
      return new Date(dateString).toLocaleString();
    }
  }
};
</script>
```

---

## Testing

### Test Assignment Creation

```bash
# Test creating an assignment
curl -X POST "http://localhost:8080/api/v1/assignments" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: test_school_123" \
  -d '{
    "class_id": "test_class_123",
    "subject_id": "test_subject_123",
    "title": "Test Assignment",
    "due_date": "2026-12-31T23:59:59Z",
    "max_score": 100,
    "status": "Published"
  }'
```

### Test Submission

```bash
# Test submitting an assignment
curl -X POST "http://localhost:8080/api/v1/assignments/ASSIGNMENT_ID/submit" \
  -H "Authorization: Bearer STUDENT_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: test_school_123" \
  -d '{
    "comment": "Test submission"
  }'
```

### Test Grading

```bash
# Test grading a submission
curl -X PUT "http://localhost:8080/api/v1/assignments/ASSIGNMENT_ID/grade/SUBMISSION_ID" \
  -H "Authorization: Bearer TEACHER_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: test_school_123" \
  -d '{
    "score": 95,
    "feedback": "Great work!"
  }'
```

---

## Troubleshooting

### Issue: "Teacher is not assigned to this subject"

**Solution:** Ensure the teacher is assigned to the subject in the `class_subjects` collection.

```bash
# Check teacher assignment
curl -X GET "http://localhost:8080/api/v1/class-subjects?field[]=teacher_id&value[]=TEACHER_ID" \
  -H "X-School-ID: YOUR_SCHOOL_ID"
```

### Issue: "Submission deadline has passed"

**Solution:** Either:
1. Update the assignment to allow late submissions
2. Extend the due date

```bash
# Allow late submissions
curl -X PUT "http://localhost:8080/api/v1/assignments/ASSIGNMENT_ID" \
  -H "Authorization: Bearer TEACHER_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: YOUR_SCHOOL_ID" \
  -d '{
    "allow_late_submission": true
  }'
```

### Issue: "You have already submitted this assignment"

**Solution:** Use the update endpoint instead:

```bash
curl -X PUT "http://localhost:8080/api/v1/submissions/SUBMISSION_ID" \
  -H "Authorization: Bearer STUDENT_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-School-ID: YOUR_SCHOOL_ID" \
  -d '{
    "comment": "Updated submission"
  }'
```

---

## Next Steps

1. **Read the full documentation**: [ASSIGNMENT_SYSTEM.md](./ASSIGNMENT_SYSTEM.md)
2. **Explore the API reference**: [API_REFERENCE_ASSIGNMENTS.md](./API_REFERENCE_ASSIGNMENTS.md)
3. **Join the developer community**: https://forum.space-together.com
4. **Report issues**: https://github.com/space-together/api/issues

---

## Support

Need help? Contact us:
- Email: support@space-together.com
- Forum: https://forum.space-together.com
- Documentation: https://docs.space-together.com
