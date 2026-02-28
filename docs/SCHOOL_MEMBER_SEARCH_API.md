# School Member Search API

## Overview
The School Member Search API allows you to search for all members within a school across different member types: students, teachers, staff, and parents.

## Endpoint

### Search School Members
```
GET /schools/{school_id}/search/members
```

Searches for members across all member types (students, teachers, staff, parents) within a specific school.

## Request Parameters

### Path Parameters
- `school_id` (required): The ID of the school to search within

### Query Parameters
All parameters from `RequestQuery` are supported:

- `filter` (optional): Search text to filter members by name, email, phone, etc.
- `limit` (optional): Maximum number of results to return (default: 20)
- `skip` (optional): Number of results to skip for pagination (default: 0)
- `field` (optional): Array of field names for custom filtering
- `value` (optional): Array of values corresponding to fields
- `class_id` (optional): Filter by class ID
- `education_year_id` (optional): Filter by education year
- `school_id` (optional): Additional school ID filter
- `term_id` (optional): Filter by term
- `exam_id` (optional): Filter by exam
- `gpa_threshold` (optional): Filter by GPA threshold
- `by_ids` (optional): Array of specific IDs to search for

## Response

Returns a paginated list of `RelatedUser` objects.

### Response Structure
```json
{
  "data": [
    {
      "user_type": "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT",
      // ... member-specific fields
    }
  ],
  "total": 100,
  "total_pages": 5,
  "current_page": 1
}
```

### RelatedUser Types

#### STUDENT
```json
{
  "user_type": "STUDENT",
  "id": "...",
  "name": "John Doe",
  "email": "john@example.com",
  "class_id": "...",
  // ... other student fields
}
```

#### TEACHER
```json
{
  "user_type": "TEACHER",
  "id": "...",
  "name": "Jane Smith",
  "email": "jane@example.com",
  "type": "Regular",
  "class_ids": ["..."],
  "subject_ids": ["..."],
  // ... other teacher fields
}
```

#### SCHOOLSTAFF
```json
{
  "user_type": "SCHOOLSTAFF",
  "id": "...",
  "name": "Bob Johnson",
  "email": "bob@example.com",
  "type": "Director",
  // ... other staff fields
}
```

#### PARENT
```json
{
  "user_type": "PARENT",
  "id": "...",
  "name": "Alice Brown",
  "email": "alice@example.com",
  "student_ids": ["..."],
  "relationship": "Mother",
  // ... other parent fields
}
```

## Examples

### Search by Name
```bash
GET /schools/507f1f77bcf86cd799439011/search/members?filter=john
```

### Search with Pagination
```bash
GET /schools/507f1f77bcf86cd799439011/search/members?filter=smith&limit=10&skip=0
```

### Search by Class
```bash
GET /schools/507f1f77bcf86cd799439011/search/members?class_id=507f1f77bcf86cd799439012
```

### Search by Multiple Fields
```bash
GET /schools/507f1f77bcf86cd799439011/search/members?field=gender&value=MALE&field=is_active&value=true
```

## Error Responses

### School Not Found
```json
{
  "message": "School not found"
}
```
Status: 404 Not Found

### School Database Not Configured
```json
{
  "message": "School database not configured"
}
```
Status: 400 Bad Request

### Invalid Query Parameters
```json
{
  "message": "Invalid query parameters"
}
```
Status: 400 Bad Request

## Notes

1. The search is performed across all member types simultaneously
2. Results are aggregated and then paginated
3. Each member type's search uses its own searchable fields:
   - Students: name, email, phone, user_id, _id, tags, gender, class_id
   - Teachers: name, email, phone, type, _id, user_id, school_id, creator_id, class_ids, subject_ids
   - Staff: name, email, tags, type, _id
   - Parents: name, email, _id, user_id, school_id, phone, gender, relationship, status

4. The `user_type` field in the response indicates the member type
5. Authentication is not required for this endpoint (public search within school context)
6. The search respects the school's database isolation

## Implementation Details

The search is implemented in:
- API Handler: `src/api/school_api.rs::search_school_members`
- Service Method: `src/services/school_service.rs::search_members`

The method:
1. Validates the school exists and has a configured database
2. Searches across all four member collections (students, teachers, staff, parents)
3. Aggregates results into a single list
4. Applies pagination to the combined results
5. Returns a `Paginated<RelatedUser>` response
