# Frontend Developer Guide: Role & Permission System

## Table of Contents
1. [Overview](#overview)
2. [Authentication Flow](#authentication-flow)
3. [Role API Reference](#role-api-reference)
4. [Permission System](#permission-system)
5. [Guard Functions](#guard-functions)
6. [Frontend Integration Examples](#frontend-integration-examples)
7. [Feature Toggles](#feature-toggles)
8. [Parent-Child Access](#parent-child-access)
9. [Error Handling](#error-handling)
10. [Testing Scenarios](#testing-scenarios)

---

## Overview

The Space Together API implements a comprehensive role-based access control (RBAC) system with:

- **Granular Permissions**: Fine-grained control over who can do what
- **Multi-Tenant Isolation**: All operations are scoped to schools
- **Permission Scopes**: Own, Class, and School-level access
- **Feature Toggles**: Enable/disable features per school
- **Parent-Child Access**: Validate parent access to student data
- **Admin Bypass**: Admins have full access to all resources

### Key Concepts

**Roles**: Collections of permissions assigned to users
- System Roles: Built-in, cannot be modified or deleted
- Custom Roles: School-specific, fully customizable

**Permissions**: Named capabilities following format `<domain>.<resource>.<action>[.<scope>]`
- Example: `assignment.create`, `submission.read.class`

**Scopes**: Define the reach of a permission
- `Own`: User's own resources only
- `Class`: Resources within assigned classes
- `School`: All resources in the school



---

## Authentication Flow

### 1. User Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "teacher@school.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "507f1f77bcf86cd799439011",
    "email": "teacher@school.com",
    "role": "TEACHER",
    "schools": ["507f1f77bcf86cd799439012"],
    "accessible_classes": ["507f1f77bcf86cd799439013"]
  }
}
```

### 2. Include Token in Requests
All protected endpoints require the JWT token in the Authorization header:

```http
GET /roles
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 3. User Roles
The system supports 5 user roles:
- `ADMIN`: Full system access
- `SCHOOLSTAFF`: School-level management
- `TEACHER`: Class-level access
- `STUDENT`: Own data access
- `PARENT`: Child data access



---

## Role API Reference

Base URL: `/roles` (no `/api/v1` prefix)

### List All Roles
```http
GET /roles?filter=teacher&limit=10&skip=0
Authorization: Bearer {token}
```

**Query Parameters:**
- `filter` (optional): Search term for name, description, role_type
- `limit` (optional): Number of results (default: 10)
- `skip` (optional): Pagination offset (default: 0)
- `school_id` (optional): Filter by school

**Response:**
```json
{
  "data": [
    {
      "_id": "507f1f77bcf86cd799439011",
      "school_id": "507f1f77bcf86cd799439012",
      "name": "Teacher Assistant",
      "description": "Can grade assignments and view student work",
      "role_type": "Custom",
      "permissions": [
        "submission.grade",
        "submission.read.class",
        "assignment.read.class"
      ],
      "is_active": true,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 1,
  "total_pages": 1,
  "current_page": 1
}
```

### Get Role by ID
```http
GET /roles/{role_id}
Authorization: Bearer {token}
```

**Response:**
```json
{
  "_id": "507f1f77bcf86cd799439011",
  "school_id": "507f1f77bcf86cd799439012",
  "name": "Teacher Assistant",
  "description": "Can grade assignments and view student work",
  "role_type": "Custom",
  "permissions": ["submission.grade", "submission.read.class"],
  "is_active": true,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```



### Create Role (Admin Only)
```http
POST /roles
Authorization: Bearer {token}
Content-Type: application/json

{
  "school_id": "507f1f77bcf86cd799439012",
  "name": "Grading Assistant",
  "description": "Can grade student submissions",
  "role_type": "Custom",
  "permissions": [
    "submission.grade",
    "submission.read.class"
  ],
  "is_active": true
}
```

**Response:** `201 Created` with role object

**Errors:**
- `403 Forbidden`: Only admins can create roles
- `400 Bad Request`: Role name already exists or validation failed

### Update Role (Admin Only)
```http
PUT /roles/{role_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Senior Grading Assistant",
  "permissions": [
    "submission.grade",
    "submission.read.class",
    "assignment.read.class"
  ]
}
```

**Response:** `200 OK` with updated role

**Errors:**
- `403 Forbidden`: Only admins can update roles
- `400 Bad Request`: Cannot modify system roles

### Delete Role (Admin Only)
```http
DELETE /roles/{role_id}
Authorization: Bearer {token}
```

**Response:** `200 OK` with deleted role

**Errors:**
- `403 Forbidden`: Only admins can delete roles
- `400 Bad Request`: Cannot delete system roles or roles with active assignments



### Assign Role to User (Admin Only)
```http
POST /roles/assign
Authorization: Bearer {token}
Content-Type: application/json

{
  "user_id": "507f1f77bcf86cd799439013",
  "role_id": "507f1f77bcf86cd799439011",
  "school_id": "507f1f77bcf86cd799439012"
}
```

**Response:** `201 Created`
```json
{
  "_id": "507f1f77bcf86cd799439014",
  "user_id": "507f1f77bcf86cd799439013",
  "role_id": "507f1f77bcf86cd799439011",
  "school_id": "507f1f77bcf86cd799439012",
  "assigned_at": "2024-01-15T10:30:00Z"
}
```

**Errors:**
- `403 Forbidden`: Only admins can assign roles
- `400 Bad Request`: Role from different school, inactive role, or already assigned

### Get Default Permissions
```http
GET /roles/permissions
```

**Response:**
```json
[
  {
    "name": "assignment.create",
    "description": "Create assignments",
    "scope": "School"
  },
  {
    "name": "assignment.read.own",
    "description": "Read own assignments",
    "scope": "Own"
  },
  {
    "name": "submission.grade",
    "description": "Grade submissions",
    "scope": "Class"
  }
]
```

### Count Roles
```http
GET /roles/count?filter=teacher
Authorization: Bearer {token}
```

**Response:**
```json
{
  "count": 5
}
```



---

## Permission System

### Permission Naming Convention

Format: `<domain>.<resource>.<action>[.<scope>]`

**Examples:**
```
assignment.create              → Create assignments (school-wide)
assignment.read.own            → Read own assignments
assignment.read.class          → Read class assignments
assignment.read.school         → Read all school assignments
assignment.update              → Update assignments
assignment.delete              → Delete assignments
submission.grade               → Grade submissions
submission.read.own            → Read own submissions
submission.read.class          → Read class submissions
parent.read.child.assignment   → Parent reads child's assignments
role.assign                    → Assign roles to users
feature.toggle                 → Toggle features
```

### Permission Scopes

#### 1. Own Scope
User can only access their own resources.

**Use Cases:**
- Students viewing their own submissions
- Teachers viewing their own profile
- Parents viewing their own account

**Example:**
```javascript
// Student viewing their own submission
if (user.role === 'STUDENT') {
  // Can only see submissions where submission.student_id === user.id
  const permissions = ['submission.read.own'];
}
```

#### 2. Class Scope
User can access resources within their assigned classes.

**Use Cases:**
- Teachers grading submissions in their classes
- Teachers viewing students in their classes
- Class-level reports and analytics

**Example:**
```javascript
// Teacher grading class submissions
if (user.role === 'TEACHER' && user.accessible_classes.includes(classId)) {
  const permissions = ['submission.grade', 'submission.read.class'];
}
```

#### 3. School Scope
User can access all resources within the school.

**Use Cases:**
- Admins managing all school data
- School staff viewing all students
- School-wide reports

**Example:**
```javascript
// Admin viewing all submissions
if (user.role === 'ADMIN') {
  const permissions = ['submission.read.school', 'assignment.read.school'];
}
```



---

## Guard Functions

The backend uses guard functions to protect API endpoints. Understanding these helps you predict API behavior.

### 1. `check_admin(user)`
**Purpose:** Only admins can access

**Used In:**
- Role management (create, update, delete, assign)
- System configuration
- School-wide settings

**Frontend Check:**
```javascript
function canAccessAdminPanel(user) {
  return user.role === 'ADMIN';
}
```

### 2. `check_admin_or_staff(user)`
**Purpose:** Admins or school staff can access

**Used In:**
- Student management
- Teacher management
- School data management

**Frontend Check:**
```javascript
function canManageSchoolData(user) {
  return user.role === 'ADMIN' || user.role === 'SCHOOLSTAFF';
}
```

### 3. `check_admin_staff_or_teacher(user)`
**Purpose:** Admins, staff, or teachers can access

**Used In:**
- Viewing students
- Viewing assignments
- Viewing exam results

**Frontend Check:**
```javascript
function canViewStudents(user) {
  return ['ADMIN', 'SCHOOLSTAFF', 'TEACHER'].includes(user.role);
}
```

### 4. `require_permission(user, school_id, permission, role_service)` (async)
**Purpose:** Check if user has specific permission

**Used In:**
- Assignment creation (`assignment.create`)
- Assignment updates (`assignment.update`)
- Assignment deletion (`assignment.delete`)

**How It Works:**
1. Admin bypass: Admins always pass
2. Looks up user's role assignments for the school
3. Checks if any assigned role has the permission
4. Returns true/false

**Frontend Check:**
```javascript
async function hasPermission(user, schoolId, permission) {
  if (user.role === 'ADMIN') return true;
  
  // Call API to check permission
  const response = await fetch(`/users/${user.id}/permissions?school_id=${schoolId}`);
  const { permissions } = await response.json();
  return permissions.includes(permission);
}
```



### 5. `require_parent_child_access(user, student_id, parent_service)` (async)
**Purpose:** Validate parent can access specific student's data

**Used In:**
- Parent viewing child's attendance
- Parent viewing child's results
- Parent viewing child's finance records
- Parent viewing child's assignment submissions

**How It Works:**
1. Admin/Staff bypass: Always allowed
2. Checks if user has PARENT role
3. Looks up parent record by user_id
4. Validates student_id is in parent's student_ids array
5. Returns true/false

**Frontend Check:**
```javascript
async function canAccessStudentData(user, studentId) {
  if (['ADMIN', 'SCHOOLSTAFF'].includes(user.role)) return true;
  if (user.role !== 'PARENT') return false;
  
  // Call API to check parent-child relationship
  const response = await fetch(`/parents/me`);
  const parent = await response.json();
  return parent.student_ids.includes(studentId);
}
```

### 6. `require_feature_enabled(school_id, feature_name, feature_service)` (async)
**Purpose:** Check if feature is enabled for school

**Used In:**
- Assignment creation (checks `assignments.enabled`)
- Advanced analytics (checks `advanced_analytics.enabled`)
- Parent portal (checks `parent_portal.enabled`)

**How It Works:**
1. Looks up school's feature configuration
2. Checks if feature is explicitly disabled
3. Default: Features are enabled unless disabled
4. Returns true/false

**Frontend Check:**
```javascript
async function isFeatureEnabled(schoolId, featureName) {
  const response = await fetch(`/schools/${schoolId}/features`);
  const { features } = await response.json();
  return features[featureName] !== false; // Default to enabled
}
```



---

## Frontend Integration Examples

### React Example: Role-Based UI

```jsx
import React, { useState, useEffect } from 'react';
import { useAuth } from './hooks/useAuth';

function RoleManagement() {
  const { user, token } = useAuth();
  const [roles, setRoles] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchRoles();
  }, []);

  const fetchRoles = async () => {
    try {
      const response = await fetch('/roles?limit=50', {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      const data = await response.json();
      setRoles(data.data);
    } catch (error) {
      console.error('Failed to fetch roles:', error);
    } finally {
      setLoading(false);
    }
  };

  const createRole = async (roleData) => {
    try {
      const response = await fetch('/roles', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(roleData)
      });

      if (response.status === 403) {
        alert('Only admins can create roles');
        return;
      }

      if (response.ok) {
        const newRole = await response.json();
        setRoles([...roles, newRole]);
      }
    } catch (error) {
      console.error('Failed to create role:', error);
    }
  };

  // Only show to admins
  if (user.role !== 'ADMIN') {
    return <div>Access Denied: Admin role required</div>;
  }

  return (
    <div>
      <h1>Role Management</h1>
      {loading ? (
        <p>Loading...</p>
      ) : (
        <RoleList roles={roles} onCreate={createRole} />
      )}
    </div>
  );
}
```



### React Example: Permission-Based Features

```jsx
import React, { useState, useEffect } from 'react';
import { useAuth } from './hooks/useAuth';

function AssignmentCreator() {
  const { user, token, schoolId } = useAuth();
  const [canCreate, setCanCreate] = useState(false);
  const [featureEnabled, setFeatureEnabled] = useState(false);

  useEffect(() => {
    checkPermissions();
  }, []);

  const checkPermissions = async () => {
    // Check if user has permission
    if (user.role === 'ADMIN') {
      setCanCreate(true);
    } else {
      // For non-admins, check if they have the permission
      // This would require a backend endpoint to check user permissions
      const hasPermission = await checkUserPermission('assignment.create');
      setCanCreate(hasPermission);
    }

    // Check if feature is enabled
    const enabled = await checkFeatureEnabled('assignments.enabled');
    setFeatureEnabled(enabled);
  };

  const checkUserPermission = async (permission) => {
    try {
      const response = await fetch(
        `/users/${user.id}/permissions?school_id=${schoolId}`,
        {
          headers: { 'Authorization': `Bearer ${token}` }
        }
      );
      const { permissions } = await response.json();
      return permissions.includes(permission);
    } catch {
      return false;
    }
  };

  const checkFeatureEnabled = async (feature) => {
    try {
      const response = await fetch(`/schools/${schoolId}/features`, {
        headers: { 'Authorization': `Bearer ${token}` }
      });
      const { features } = await response.json();
      return features[feature] !== false;
    } catch {
      return true; // Default to enabled
    }
  };

  const createAssignment = async (assignmentData) => {
    try {
      const response = await fetch('/assignments', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(assignmentData)
      });

      if (response.status === 403) {
        const error = await response.json();
        alert(error.message || 'Permission denied');
        return;
      }

      if (response.ok) {
        const assignment = await response.json();
        console.log('Created:', assignment);
      }
    } catch (error) {
      console.error('Failed to create assignment:', error);
    }
  };

  if (!featureEnabled) {
    return <div>Assignments feature is disabled for this school</div>;
  }

  if (!canCreate) {
    return <div>You don't have permission to create assignments</div>;
  }

  return (
    <div>
      <h1>Create Assignment</h1>
      {/* Assignment form */}
    </div>
  );
}
```



### Vue Example: Parent-Child Access

```vue
<template>
  <div>
    <h1>My Children's Assignments</h1>
    
    <div v-if="loading">Loading...</div>
    
    <div v-else-if="!canAccess">
      Access Denied: You don't have access to this student
    </div>
    
    <div v-else>
      <StudentAssignments :student-id="studentId" />
    </div>
  </div>
</template>

<script>
export default {
  name: 'ParentStudentView',
  props: ['studentId'],
  data() {
    return {
      loading: true,
      canAccess: false,
      user: null,
      token: null
    };
  },
  async mounted() {
    this.user = this.$store.state.auth.user;
    this.token = this.$store.state.auth.token;
    await this.checkAccess();
  },
  methods: {
    async checkAccess() {
      try {
        // Admin and staff always have access
        if (['ADMIN', 'SCHOOLSTAFF'].includes(this.user.role)) {
          this.canAccess = true;
          this.loading = false;
          return;
        }

        // Check parent-child relationship
        if (this.user.role === 'PARENT') {
          const response = await fetch('/parents/me', {
            headers: {
              'Authorization': `Bearer ${this.token}`
            }
          });
          
          const parent = await response.json();
          this.canAccess = parent.student_ids.includes(this.studentId);
        }
      } catch (error) {
        console.error('Failed to check access:', error);
        this.canAccess = false;
      } finally {
        this.loading = false;
      }
    }
  }
};
</script>
```



### Angular Example: Role Guard Service

```typescript
// auth.service.ts
import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private token: string = '';
  private user: any = null;

  constructor(private http: HttpClient) {}

  getHeaders(): HttpHeaders {
    return new HttpHeaders({
      'Authorization': `Bearer ${this.token}`,
      'Content-Type': 'application/json'
    });
  }

  isAdmin(): boolean {
    return this.user?.role === 'ADMIN';
  }

  isAdminOrStaff(): boolean {
    return ['ADMIN', 'SCHOOLSTAFF'].includes(this.user?.role);
  }

  isTeacher(): boolean {
    return this.user?.role === 'TEACHER';
  }

  isParent(): boolean {
    return this.user?.role === 'PARENT';
  }

  hasPermission(permission: string, schoolId: string): Observable<boolean> {
    if (this.isAdmin()) {
      return new Observable(observer => {
        observer.next(true);
        observer.complete();
      });
    }

    return this.http.get<{ permissions: string[] }>(
      `/users/${this.user.id}/permissions?school_id=${schoolId}`,
      { headers: this.getHeaders() }
    ).pipe(
      map(response => response.permissions.includes(permission))
    );
  }
}

// role.service.ts
import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { AuthService } from './auth.service';

@Injectable({
  providedIn: 'root'
})
export class RoleService {
  constructor(
    private http: HttpClient,
    private auth: AuthService
  ) {}

  getRoles(filter?: string, limit = 10, skip = 0): Observable<any> {
    const params = new URLSearchParams();
    if (filter) params.append('filter', filter);
    params.append('limit', limit.toString());
    params.append('skip', skip.toString());

    return this.http.get(`/roles?${params.toString()}`, {
      headers: this.auth.getHeaders()
    });
  }

  createRole(roleData: any): Observable<any> {
    return this.http.post('/roles', roleData, {
      headers: this.auth.getHeaders()
    });
  }

  assignRole(userId: string, roleId: string, schoolId: string): Observable<any> {
    return this.http.post('/roles/assign', 
      { user_id: userId, role_id: roleId, school_id: schoolId },
      { headers: this.auth.getHeaders() }
    );
  }
}
```



---

## Feature Toggles

Feature toggles allow schools to enable/disable features dynamically.

### Common Features

```javascript
const FEATURES = {
  ASSIGNMENTS: 'assignments.enabled',
  PARENT_PORTAL: 'parent_portal.enabled',
  ADVANCED_ANALYTICS: 'advanced_analytics.enabled',
  ONLINE_PAYMENTS: 'online_payments.enabled',
  MESSAGING: 'messaging.enabled'
};
```

### Check Feature Status

```javascript
async function checkFeature(schoolId, featureName) {
  try {
    const response = await fetch(`/schools/${schoolId}/features`, {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    });
    
    const { features } = await response.json();
    
    // Features are enabled by default unless explicitly disabled
    return features[featureName] !== false;
  } catch (error) {
    console.error('Failed to check feature:', error);
    return true; // Default to enabled on error
  }
}
```

### React Hook for Feature Toggles

```jsx
import { useState, useEffect } from 'react';
import { useAuth } from './useAuth';

export function useFeature(featureName) {
  const { schoolId, token } = useAuth();
  const [enabled, setEnabled] = useState(true);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function checkFeature() {
      try {
        const response = await fetch(`/schools/${schoolId}/features`, {
          headers: { 'Authorization': `Bearer ${token}` }
        });
        const { features } = await response.json();
        setEnabled(features[featureName] !== false);
      } catch (error) {
        console.error('Feature check failed:', error);
        setEnabled(true); // Default to enabled
      } finally {
        setLoading(false);
      }
    }

    checkFeature();
  }, [schoolId, featureName, token]);

  return { enabled, loading };
}

// Usage
function AssignmentPage() {
  const { enabled, loading } = useFeature('assignments.enabled');

  if (loading) return <div>Loading...</div>;
  if (!enabled) return <div>Assignments feature is disabled</div>;

  return <div>Assignment content...</div>;
}
```



---

## Parent-Child Access

Parents can only access data for their registered children.

### Parent Data Structure

```json
{
  "_id": "507f1f77bcf86cd799439011",
  "user_id": "507f1f77bcf86cd799439012",
  "school_id": "507f1f77bcf86cd799439013",
  "student_ids": [
    "507f1f77bcf86cd799439014",
    "507f1f77bcf86cd799439015"
  ],
  "first_name": "John",
  "last_name": "Doe",
  "email": "john.doe@email.com",
  "phone": "+1234567890"
}
```

### Endpoints Using Parent-Child Guard

1. **Parent Attendance** - `GET /parents/attendance/{student_id}`
2. **Parent Results** - `GET /parents/results/{student_id}`
3. **Parent Finance** - `GET /parents/finance/{student_id}`
4. **Parent Assignment Submissions** - `GET /assignments/{assignment_id}/submissions?student_id={student_id}`

### Frontend Implementation

```javascript
class ParentService {
  constructor(token) {
    this.token = token;
  }

  async getMyChildren() {
    const response = await fetch('/parents/me', {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    const parent = await response.json();
    return parent.student_ids;
  }

  async canAccessStudent(studentId) {
    const children = await this.getMyChildren();
    return children.includes(studentId);
  }

  async getStudentAttendance(studentId) {
    // Check access first
    if (!await this.canAccessStudent(studentId)) {
      throw new Error('Access denied: Not your child');
    }

    const response = await fetch(`/parents/attendance/${studentId}`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });

    if (response.status === 403) {
      throw new Error('Access denied');
    }

    return await response.json();
  }

  async getStudentResults(studentId) {
    const response = await fetch(`/parents/results/${studentId}`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });

    if (response.status === 403) {
      throw new Error('Access denied: You do not have access to this student');
    }

    return await response.json();
  }

  async getStudentSubmissions(assignmentId, studentId) {
    const response = await fetch(
      `/assignments/${assignmentId}/submissions?student_id=${studentId}`,
      {
        headers: { 'Authorization': `Bearer ${this.token}` }
      }
    );

    if (response.status === 403) {
      throw new Error('Access denied');
    }

    return await response.json();
  }
}
```



---

## Error Handling

### Common HTTP Status Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Validation error or invalid data
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

### Error Response Format

```json
{
  "message": "Access denied: assignment.create permission required"
}
```

### Frontend Error Handler

```javascript
class APIError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
    this.name = 'APIError';
  }
}

async function apiRequest(url, options = {}) {
  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      const error = await response.json();
      throw new APIError(response.status, error.message || 'Request failed');
    }

    return await response.json();
  } catch (error) {
    if (error instanceof APIError) {
      throw error;
    }
    throw new APIError(500, 'Network error');
  }
}

// Usage
try {
  const role = await apiRequest('/roles', {
    method: 'POST',
    body: JSON.stringify(roleData)
  });
  console.log('Created:', role);
} catch (error) {
  if (error.status === 403) {
    alert('Permission denied: Only admins can create roles');
  } else if (error.status === 400) {
    alert(`Validation error: ${error.message}`);
  } else {
    alert('An error occurred. Please try again.');
  }
}
```

### React Error Boundary

```jsx
import React from 'react';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    console.error('Error caught:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-container">
          <h1>Something went wrong</h1>
          <p>{this.state.error?.message}</p>
          <button onClick={() => window.location.reload()}>
            Reload Page
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
```



---

## Testing Scenarios

### 1. Role Management Tests

```javascript
describe('Role Management', () => {
  test('Admin can create role', async () => {
    const adminToken = await loginAsAdmin();
    const role = await createRole(adminToken, {
      name: 'Test Role',
      school_id: schoolId,
      permissions: ['assignment.create']
    });
    expect(role._id).toBeDefined();
  });

  test('Non-admin cannot create role', async () => {
    const teacherToken = await loginAsTeacher();
    await expect(
      createRole(teacherToken, { name: 'Test Role' })
    ).rejects.toThrow('403');
  });

  test('Cannot delete system role', async () => {
    const adminToken = await loginAsAdmin();
    const systemRole = await findSystemRole();
    await expect(
      deleteRole(adminToken, systemRole._id)
    ).rejects.toThrow('Cannot delete system roles');
  });

  test('Cannot delete role with assignments', async () => {
    const adminToken = await loginAsAdmin();
    const role = await createRoleWithAssignment();
    await expect(
      deleteRole(adminToken, role._id)
    ).rejects.toThrow('It is assigned to');
  });
});
```

### 2. Permission Tests

```javascript
describe('Permission Checks', () => {
  test('Admin bypasses permission check', async () => {
    const adminToken = await loginAsAdmin();
    const assignment = await createAssignment(adminToken, assignmentData);
    expect(assignment._id).toBeDefined();
  });

  test('User with permission can create assignment', async () => {
    const userToken = await loginAsUserWithPermission('assignment.create');
    const assignment = await createAssignment(userToken, assignmentData);
    expect(assignment._id).toBeDefined();
  });

  test('User without permission cannot create assignment', async () => {
    const userToken = await loginAsUserWithoutPermission();
    await expect(
      createAssignment(userToken, assignmentData)
    ).rejects.toThrow('403');
  });
});
```

### 3. Parent-Child Access Tests

```javascript
describe('Parent-Child Access', () => {
  test('Parent can access own child data', async () => {
    const parentToken = await loginAsParent();
    const childId = await getParentChildId(parentToken);
    const attendance = await getStudentAttendance(parentToken, childId);
    expect(attendance).toBeDefined();
  });

  test('Parent cannot access non-child data', async () => {
    const parentToken = await loginAsParent();
    const otherStudentId = await getOtherStudentId();
    await expect(
      getStudentAttendance(parentToken, otherStudentId)
    ).rejects.toThrow('403');
  });

  test('Admin can access any student data', async () => {
    const adminToken = await loginAsAdmin();
    const studentId = await getRandomStudentId();
    const attendance = await getStudentAttendance(adminToken, studentId);
    expect(attendance).toBeDefined();
  });
});
```

### 4. Feature Toggle Tests

```javascript
describe('Feature Toggles', () => {
  test('Can create assignment when feature enabled', async () => {
    await enableFeature(schoolId, 'assignments.enabled');
    const token = await loginAsTeacher();
    const assignment = await createAssignment(token, assignmentData);
    expect(assignment._id).toBeDefined();
  });

  test('Cannot create assignment when feature disabled', async () => {
    await disableFeature(schoolId, 'assignments.enabled');
    const token = await loginAsTeacher();
    await expect(
      createAssignment(token, assignmentData)
    ).rejects.toThrow('Feature');
  });
});
```



---

## Best Practices

### 1. Always Check Permissions Client-Side
Don't rely solely on backend validation. Check permissions before showing UI elements.

```javascript
// Good
if (user.role === 'ADMIN') {
  return <CreateRoleButton />;
}

// Bad - showing button that will fail
return <CreateRoleButton />;
```

### 2. Handle 403 Errors Gracefully
Show user-friendly messages when access is denied.

```javascript
try {
  await createRole(roleData);
} catch (error) {
  if (error.status === 403) {
    showNotification('You don\'t have permission to create roles', 'error');
  }
}
```

### 3. Cache Permission Checks
Don't check permissions on every render. Cache results.

```javascript
const [permissions, setPermissions] = useState(new Set());

useEffect(() => {
  async function loadPermissions() {
    const perms = await fetchUserPermissions(user.id, schoolId);
    setPermissions(new Set(perms));
  }
  loadPermissions();
}, [user.id, schoolId]);

// Fast lookup
const canCreate = permissions.has('assignment.create');
```

### 4. Use Role-Based Routing
Protect routes based on user roles.

```javascript
// React Router example
<Route 
  path="/admin/roles" 
  element={
    <ProtectedRoute requiredRole="ADMIN">
      <RoleManagement />
    </ProtectedRoute>
  } 
/>
```

### 5. Show Loading States
Always show loading states while checking permissions.

```javascript
if (loading) return <Spinner />;
if (!hasPermission) return <AccessDenied />;
return <Content />;
```

### 6. Multi-Tenant Awareness
Always include school_id in requests.

```javascript
// Good
await fetch(`/roles?school_id=${schoolId}`);

// Bad - might return roles from other schools
await fetch('/roles');
```

### 7. Refresh Permissions on Role Changes
When a user's role is updated, refresh their permissions.

```javascript
async function onRoleAssigned(userId) {
  if (userId === currentUser.id) {
    await refreshUserPermissions();
    showNotification('Your permissions have been updated');
  }
}
```

---

## Quick Reference

### User Roles
- `ADMIN` - Full system access
- `SCHOOLSTAFF` - School management
- `TEACHER` - Class-level access
- `STUDENT` - Own data only
- `PARENT` - Child data only

### Permission Scopes
- `Own` - User's own resources
- `Class` - Resources in assigned classes
- `School` - All school resources

### Common Permissions
- `assignment.create` - Create assignments
- `assignment.update` - Update assignments
- `assignment.delete` - Delete assignments
- `submission.grade` - Grade submissions
- `role.assign` - Assign roles
- `feature.toggle` - Toggle features

### API Endpoints
- `GET /roles` - List roles
- `POST /roles` - Create role (Admin)
- `PUT /roles/{id}` - Update role (Admin)
- `DELETE /roles/{id}` - Delete role (Admin)
- `POST /roles/assign` - Assign role (Admin)
- `GET /roles/permissions` - Get default permissions

---

## Support

For questions or issues:
1. Check this documentation
2. Review the backend implementation docs
3. Contact the backend team
4. Check API response error messages

