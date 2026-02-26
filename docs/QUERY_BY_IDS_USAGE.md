# Query Students by Multiple IDs

## Overview

You can now query multiple students by their IDs using the `by_ids` query parameter. This works with all existing student endpoints.

## Usage

### Basic Query

Get multiple students by their IDs:

```http
GET /students?by_ids=507f1f77bcf86cd799439011&by_ids=507f191e810c19729de860ea&by_ids=507f1f77bcf86cd799439012
```

### With Relations

Get students with populated relations (user, school, class, etc.):

```http
GET /students/others?by_ids=507f1f77bcf86cd799439011&by_ids=507f191e810c19729de860ea
```

### Combined with Other Filters

You can combine `by_ids` with other query parameters:

```http
GET /students?by_ids=507f1f77bcf86cd799439011&by_ids=507f191e810c19729de860ea&class_id=507f1f77bcf86cd799439013
```

This will return students that match BOTH conditions:
- Student ID is in the provided list
- AND student belongs to the specified class

### With Pagination

```http
GET /students?by_ids=507f1f77bcf86cd799439011&by_ids=507f191e810c19729de860ea&limit=10&skip=0
```

### With Search Filter

```http
GET /students?by_ids=507f1f77bcf86cd799439011&by_ids=507f191e810c19729de860ea&filter=john
```

## Frontend Examples

### JavaScript/TypeScript

```typescript
// Get multiple students by IDs
async function getStudentsByIds(ids: string[]) {
  const params = new URLSearchParams();
  ids.forEach(id => params.append('by_ids', id));
  
  const response = await fetch(`/api/v1/students?${params.toString()}`, {
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-School-Token': schoolToken
    }
  });
  
  return response.json();
}

// Usage
const studentIds = [
  '507f1f77bcf86cd799439011',
  '507f191e810c19729de860ea',
  '507f1f77bcf86cd799439012'
];

const students = await getStudentsByIds(studentIds);
console.log(students.data); // Array of students
```

### With Relations

```typescript
async function getStudentsByIdsWithRelations(ids: string[]) {
  const params = new URLSearchParams();
  ids.forEach(id => params.append('by_ids', id));
  
  const response = await fetch(`/api/v1/students/others?${params.toString()}`, {
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-School-Token': schoolToken
    }
  });
  
  return response.json();
}
```

### React Hook Example

```typescript
import { useState, useEffect } from 'react';

function useStudentsByIds(ids: string[]) {
  const [students, setStudents] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (ids.length === 0) {
      setStudents([]);
      setLoading(false);
      return;
    }

    const params = new URLSearchParams();
    ids.forEach(id => params.append('by_ids', id));

    fetch(`/api/v1/students?${params.toString()}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'X-School-Token': schoolToken
      }
    })
      .then(res => res.json())
      .then(data => {
        setStudents(data.data);
        setLoading(false);
      })
      .catch(err => {
        setError(err);
        setLoading(false);
      });
  }, [ids.join(',')]); // Re-fetch when IDs change

  return { students, loading, error };
}

// Usage in component
function StudentList({ studentIds }) {
  const { students, loading, error } = useStudentsByIds(studentIds);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <ul>
      {students.map(student => (
        <li key={student._id}>{student.name}</li>
      ))}
    </ul>
  );
}
```

### Axios Example

```typescript
import axios from 'axios';

async function getStudentsByIds(ids: string[]) {
  const response = await axios.get('/api/v1/students', {
    params: {
      by_ids: ids
    },
    paramsSerializer: params => {
      // Properly serialize array parameters
      return Object.entries(params)
        .flatMap(([key, value]) => 
          Array.isArray(value) 
            ? value.map(v => `${key}=${encodeURIComponent(v)}`)
            : `${key}=${encodeURIComponent(value)}`
        )
        .join('&');
    },
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-School-Token': schoolToken
    }
  });
  
  return response.data;
}
```

## Response Format

### Without Relations

```json
{
  "data": [
    {
      "_id": "507f1f77bcf86cd799439011",
      "name": "John Doe",
      "email": "john@example.com",
      "class_id": "507f1f77bcf86cd799439013",
      "school_id": "507f1f77bcf86cd799439014",
      "status": "Active",
      "created_at": "2024-01-01T00:00:00Z"
    },
    {
      "_id": "507f191e810c19729de860ea",
      "name": "Jane Smith",
      "email": "jane@example.com",
      "class_id": "507f1f77bcf86cd799439013",
      "school_id": "507f1f77bcf86cd799439014",
      "status": "Active",
      "created_at": "2024-01-02T00:00:00Z"
    }
  ],
  "total": 2,
  "total_pages": 1,
  "current_page": 1
}
```

### With Relations

```json
{
  "data": [
    {
      "student": {
        "_id": "507f1f77bcf86cd799439011",
        "name": "John Doe",
        "email": "john@example.com"
      },
      "user": {
        "_id": "507f1f77bcf86cd799439015",
        "username": "johndoe"
      },
      "class": {
        "_id": "507f1f77bcf86cd799439013",
        "name": "Grade 10A"
      },
      "school": {
        "_id": "507f1f77bcf86cd799439014",
        "name": "Example School"
      }
    }
  ],
  "total": 1,
  "total_pages": 1,
  "current_page": 1
}
```

## Error Handling

### No Valid IDs

If all provided IDs are invalid:

```json
{
  "message": "No valid IDs provided in by_ids parameter"
}
```

### Empty Result

If no students match the provided IDs:

```json
{
  "data": [],
  "total": 0,
  "total_pages": 1,
  "current_page": 1
}
```

## Implementation Details

The `by_ids` parameter:
- Accepts multiple values (repeat the parameter for each ID)
- Automatically converts string IDs to MongoDB ObjectIds
- Filters out invalid IDs (non-ObjectId format)
- Uses MongoDB `$in` operator for efficient querying
- Can be combined with other filters using `$and` logic
- Works with all existing endpoints (with/without relations)

## Performance Considerations

- The query uses MongoDB's `$in` operator which is indexed
- Recommended to limit the number of IDs to 100 per request
- For large batches, consider pagination
- Invalid IDs are silently filtered out (no error thrown)

## Use Cases

1. **Bulk Student Display**: Show details for multiple selected students
2. **Class Roster**: Get all students in a class by their IDs
3. **Report Generation**: Fetch specific students for report cards
4. **Parent Portal**: Show all children of a parent
5. **Attendance**: Get students for attendance marking
6. **Grade Entry**: Fetch students for grade input

## Notes

- IDs must be valid MongoDB ObjectId format (24 hex characters)
- Invalid IDs are automatically filtered out
- Empty `by_ids` array returns all students (no filter applied)
- Combines with other filters using AND logic
- Respects tenant isolation (school_id filtering)
- Respects soft delete (deleted students not returned)
