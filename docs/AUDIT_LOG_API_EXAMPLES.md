# Audit Log API - Request/Response Examples

## Authentication

All requests require a valid JWT token in the Authorization header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Base URL

```
Production: https://api.yourschool.com/api/audit-logs
Development: http://localhost:8080/api/audit-logs
```

---

## 1. List All Audit Logs (Paginated)

### Request

```http
GET /api/audit-logs?limit=20&skip=0 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
Content-Type: application/json
```

### cURL

```bash
curl -X GET "https://api.yourschool.com/api/audit-logs?limit=20&skip=0" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json"
```

### JavaScript (Fetch)

```javascript
const response = await fetch('https://api.yourschool.com/api/audit-logs?limit=20&skip=0', {
  method: 'GET',
  headers: {
    'Authorization': 'Bearer YOUR_TOKEN_HERE',
    'Content-Type': 'application/json'
  }
});

const data = await response.json();
```

### Axios

```javascript
import axios from 'axios';

const response = await axios.get('https://api.yourschool.com/api/audit-logs', {
  params: {
    limit: 20,
    skip: 0
  },
  headers: {
    'Authorization': 'Bearer YOUR_TOKEN_HERE'
  }
});

const data = response.data;
```

### Response (200 OK)

```json
{
  "data": [
    {
      "_id": "65f8a1b2c3d4e5f6a7b8c9d0",
      "school_id": "65f8a1b2c3d4e5f6a7b8c9d1",
      "user_id": "65f8a1b2c3d4e5f6a7b8c9d2",
      "user_role": "TEACHER",
      "action": "submission.grade.update",
      "entity_type": "submission",
      "entity_id": "65f8a1b2c3d4e5f6a7b8c9d3",
      "metadata": {
        "before_score": 75,
        "after_score": 85,
        "feedback_provided": true
      },
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
      "severity": "INFO",
      "created_at": "2024-03-15T10:30:00.000Z"
    },
    {
      "_id": "65f8a1b2c3d4e5f6a7b8c9d4",
      "school_id": "65f8a1b2c3d4e5f6a7b8c9d1",
      "user_id": "65f8a1b2c3d4e5f6a7b8c9d5",
      "user_role": "ADMIN",
      "action": "student.delete",
      "entity_type": "student",
      "entity_id": "65f8a1b2c3d4e5f6a7b8c9d6",
      "metadata": {
        "student_name": "John Doe",
        "student_email": "john.doe@example.com"
      },
      "ip_address": "192.168.1.101",
      "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)...",
      "severity": "CRITICAL",
      "created_at": "2024-03-15T09:15:00.000Z"
    }
  ],
  "total": 156,
  "total_pages": 8,
  "current_page": 1
}
```

---

## 2. List Audit Logs with Relations

### Request

```http
GET /api/audit-logs/others?limit=10&skip=0 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const response = await fetch('https://api.yourschool.com/api/audit-logs/others?limit=10', {
  headers: {
    'Authorization': 'Bearer YOUR_TOKEN_HERE'
  }
});

const data = await response.json();
```

### Response (200 OK)

```json
{
  "data": [
    {
      "_id": "65f8a1b2c3d4e5f6a7b8c9d0",
      "school_id": "65f8a1b2c3d4e5f6a7b8c9d1",
      "user_id": "65f8a1b2c3d4e5f6a7b8c9d2",
      "user_role": "TEACHER",
      "action": "submission.grade.update",
      "entity_type": "submission",
      "entity_id": "65f8a1b2c3d4e5f6a7b8c9d3",
      "metadata": {
        "before_score": 75,
        "after_score": 85
      },
      "severity": "INFO",
      "created_at": "2024-03-15T10:30:00.000Z",
      "user": {
        "_id": "65f8a1b2c3d4e5f6a7b8c9d2",
        "name": "Jane Smith",
        "email": "jane.smith@school.com",
        "image": "https://cloudinary.com/image.jpg",
        "role": "TEACHER"
      },
      "school": {
        "_id": "65f8a1b2c3d4e5f6a7b8c9d1",
        "name": "Springfield High School",
        "email": "admin@springfield.edu"
      }
    }
  ],
  "total": 156,
  "total_pages": 16,
  "current_page": 1
}
```

---

## 3. Filter by Severity

### Request

```http
GET /api/audit-logs?severity=CRITICAL&limit=50 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const response = await fetch(
  'https://api.yourschool.com/api/audit-logs?severity=CRITICAL&limit=50',
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

### Response

Returns only logs with severity = "CRITICAL"

---

## 4. Filter by Action Type

### Request

```http
GET /api/audit-logs?action=submission.grade.update&limit=30 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const response = await fetch(
  'https://api.yourschool.com/api/audit-logs?action=submission.grade.update&limit=30',
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

---

## 5. Filter by User ID

### Request

```http
GET /api/audit-logs/others?user_id=65f8a1b2c3d4e5f6a7b8c9d2&limit=20 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const userId = '65f8a1b2c3d4e5f6a7b8c9d2';
const response = await fetch(
  `https://api.yourschool.com/api/audit-logs/others?user_id=${userId}&limit=20`,
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

---

## 6. Filter by Date Range

### Request

```http
GET /api/audit-logs?from_date=2024-01-01&to_date=2024-03-31&limit=100 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const params = new URLSearchParams({
  from_date: '2024-01-01',
  to_date: '2024-03-31',
  limit: '100'
});

const response = await fetch(
  `https://api.yourschool.com/api/audit-logs?${params}`,
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

---

## 7. Filter by Entity Type

### Request

```http
GET /api/audit-logs?entity_type=student&limit=50 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const response = await fetch(
  'https://api.yourschool.com/api/audit-logs?entity_type=student&limit=50',
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

---

## 8. Search Audit Logs

### Request

```http
GET /api/audit-logs?filter=john&limit=20 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const searchTerm = 'john';
const response = await fetch(
  `https://api.yourschool.com/api/audit-logs?filter=${encodeURIComponent(searchTerm)}&limit=20`,
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

---

## 9. Get Single Audit Log by ID

### Request

```http
GET /api/audit-logs/65f8a1b2c3d4e5f6a7b8c9d0 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const auditLogId = '65f8a1b2c3d4e5f6a7b8c9d0';
const response = await fetch(
  `https://api.yourschool.com/api/audit-logs/${auditLogId}`,
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);

const auditLog = await response.json();
```

### Response (200 OK)

```json
{
  "_id": "65f8a1b2c3d4e5f6a7b8c9d0",
  "school_id": "65f8a1b2c3d4e5f6a7b8c9d1",
  "user_id": "65f8a1b2c3d4e5f6a7b8c9d2",
  "user_role": "TEACHER",
  "action": "submission.grade.update",
  "entity_type": "submission",
  "entity_id": "65f8a1b2c3d4e5f6a7b8c9d3",
  "metadata": {
    "before_score": 75,
    "after_score": 85,
    "feedback_provided": true
  },
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "severity": "INFO",
  "created_at": "2024-03-15T10:30:00.000Z"
}
```

### Response (404 Not Found)

```json
{
  "message": "Audit log not found"
}
```

---

## 10. Get Single Audit Log with Relations

### Request

```http
GET /api/audit-logs/65f8a1b2c3d4e5f6a7b8c9d0/others HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const auditLogId = '65f8a1b2c3d4e5f6a7b8c9d0';
const response = await fetch(
  `https://api.yourschool.com/api/audit-logs/${auditLogId}/others`,
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);

const auditLog = await response.json();
console.log(auditLog.user.name); // Access user name directly
```

---

## 11. Find One by Match Criteria

### Request

```http
GET /api/audit-logs/match?entity_type=student&action=student.delete HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const response = await fetch(
  'https://api.yourschool.com/api/audit-logs/match?entity_type=student&action=student.delete',
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);

const firstMatch = await response.json();
```

---

## 12. Count Audit Logs

### Request

```http
GET /api/audit-logs/count?severity=CRITICAL HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const response = await fetch(
  'https://api.yourschool.com/api/audit-logs/count?severity=CRITICAL',
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);

const { count } = await response.json();
console.log(`Total critical operations: ${count}`);
```

### Response (200 OK)

```json
{
  "count": 42
}
```

---

## 13. Complex Filter Example

### Request

Combine multiple filters:

```http
GET /api/audit-logs/others?severity=CRITICAL&entity_type=student&from_date=2024-01-01&to_date=2024-12-31&limit=50&skip=0 HTTP/1.1
Host: api.yourschool.com
Authorization: Bearer YOUR_TOKEN_HERE
```

### JavaScript

```javascript
const params = new URLSearchParams({
  severity: 'CRITICAL',
  entity_type: 'student',
  from_date: '2024-01-01',
  to_date: '2024-12-31',
  limit: '50',
  skip: '0'
});

const response = await fetch(
  `https://api.yourschool.com/api/audit-logs/others?${params}`,
  {
    headers: { 'Authorization': 'Bearer YOUR_TOKEN_HERE' }
  }
);
```

---

## Error Responses

### 401 Unauthorized

```json
{
  "message": "Unauthorized"
}
```

**Cause:** Missing or invalid JWT token

**Solution:** Include valid Authorization header

### 403 Forbidden

```json
{
  "message": "Access denied: audit.view permission required"
}
```

**Cause:** User doesn't have permission to view audit logs

**Solution:** Ensure user has ADMIN or SCHOOLSTAFF role

### 400 Bad Request

```json
{
  "message": "Invalid object ID"
}
```

**Cause:** Invalid ID format in request

**Solution:** Ensure IDs are valid MongoDB ObjectIds

---

## TypeScript Types

```typescript
// Request types
interface AuditLogQueryParams {
  limit?: number;
  skip?: number;
  filter?: string;
  user_id?: string;
  entity_type?: string;
  action?: string;
  entity_id?: string;
  severity?: 'INFO' | 'WARNING' | 'CRITICAL';
  from_date?: string;
  to_date?: string;
}

// Response types
interface AuditLog {
  _id: string;
  school_id: string;
  user_id: string;
  user_role: 'ADMIN' | 'TEACHER' | 'STUDENT' | 'SCHOOLSTAFF' | 'PARENT';
  action: string;
  entity_type: string;
  entity_id: string;
  metadata?: Record<string, any>;
  ip_address?: string;
  user_agent?: string;
  severity: 'INFO' | 'WARNING' | 'CRITICAL';
  created_at: string;
}

interface AuditLogWithRelations extends AuditLog {
  user?: {
    _id: string;
    name: string;
    email: string;
    image?: string;
    role?: string;
  };
  school?: {
    _id: string;
    name: string;
    email?: string;
  };
}

interface AuditLogListResponse {
  data: AuditLog[];
  total: number;
  total_pages: number;
  current_page: number;
}

interface CountResponse {
  count: number;
}
```

---

## Rate Limiting

No specific rate limits are enforced, but best practices:
- Use pagination (limit results to 20-100 per request)
- Cache results when possible
- Avoid polling - use reasonable intervals (30s+)

---

## Notes

1. All dates are in ISO 8601 format (UTC)
2. ObjectIds are 24-character hexadecimal strings
3. Metadata structure varies by action type
4. IP addresses and user agents may be null for system operations
5. Results are sorted by created_at descending (newest first)
