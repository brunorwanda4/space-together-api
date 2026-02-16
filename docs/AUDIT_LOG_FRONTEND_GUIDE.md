# Audit Log System - Frontend Developer Guide

## Overview

The Audit Log system tracks all critical operations in the application. This guide explains how to integrate audit log viewing into your frontend application.

## 🔐 Access Control

**Who can view audit logs:**
- ✅ ADMIN users
- ✅ SCHOOLSTAFF users
- ❌ TEACHER users (no access)
- ❌ STUDENT users (no access)
- ❌ PARENT users (no access)

**Required Permission:** `audit.view`

## 📡 API Endpoints

Base URL: `/api/audit-logs` or `/audit-logs`

### 1. List All Audit Logs

**Endpoint:** `GET /audit-logs`

**Query Parameters:**
```typescript
interface AuditLogQueryParams {
  limit?: number;        // Records per page (default: 10)
  skip?: number;         // Records to skip (for pagination)
  filter?: string;       // Search text (searches across multiple fields)
  user_id?: string;      // Filter by user ObjectId
  entity_type?: string;  // Filter by entity (e.g., "submission", "student")
  action?: string;       // Filter by action (e.g., "submission.grade.update")
  entity_id?: string;    // Filter by specific entity ObjectId
  severity?: 'INFO' | 'WARNING' | 'CRITICAL';
  from_date?: string;    // ISO date string (e.g., "2024-01-01")
  to_date?: string;      // ISO date string (e.g., "2024-12-31")
}
```

**Response:**
```typescript
interface AuditLogListResponse {
  data: AuditLog[];
  total: number;
  total_pages: number;
  current_page: number;
}

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
  created_at: string; // ISO date string
}
```

**Example Request:**
```javascript
// Fetch audit logs with pagination
const response = await fetch('/api/audit-logs?limit=20&skip=0', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
});

const data = await response.json();
console.log(data.data); // Array of audit logs
console.log(data.total); // Total count
```

### 2. List Audit Logs with Relations

**Endpoint:** `GET /audit-logs/others`

Same parameters as above, but includes populated user and school objects.

**Response:**
```typescript
interface AuditLogWithRelations {
  _id: string;
  school_id: string;
  user_id: string;
  user_role: string;
  action: string;
  entity_type: string;
  entity_id: string;
  metadata?: Record<string, any>;
  ip_address?: string;
  user_agent?: string;
  severity: 'INFO' | 'WARNING' | 'CRITICAL';
  created_at: string;
  
  // Populated relations
  user?: {
    _id: string;
    name: string;
    email: string;
    image?: string;
    // ... other user fields
  };
  school?: {
    _id: string;
    name: string;
    // ... other school fields
  };
}
```

**Example Request:**
```javascript
// Fetch with user and school details
const response = await fetch('/api/audit-logs/others?limit=20', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

const data = await response.json();
// Now you can access data.data[0].user.name directly
```

### 3. Get Single Audit Log by ID

**Endpoint:** `GET /audit-logs/{id}`

**Example Request:**
```javascript
const auditLogId = '507f1f77bcf86cd799439011';
const response = await fetch(`/api/audit-logs/${auditLogId}`, {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

const auditLog = await response.json();
```

### 4. Get Single Audit Log with Relations

**Endpoint:** `GET /audit-logs/{id}/others`

**Example Request:**
```javascript
const response = await fetch(`/api/audit-logs/${auditLogId}/others`, {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

const auditLog = await response.json();
console.log(auditLog.user.name); // User who performed the action
```

### 5. Find One by Match Criteria

**Endpoint:** `GET /audit-logs/match`

Finds the first audit log matching the criteria.

**Example Request:**
```javascript
// Find the most recent student deletion
const response = await fetch(
  '/api/audit-logs/match?entity_type=student&action=student.delete',
  {
    headers: { 'Authorization': `Bearer ${token}` }
  }
);
```

### 6. Count Audit Logs

**Endpoint:** `GET /audit-logs/count`

**Response:**
```typescript
interface CountResponse {
  count: number;
}
```

**Example Request:**
```javascript
// Count all critical operations
const response = await fetch('/api/audit-logs/count?severity=CRITICAL', {
  headers: { 'Authorization': `Bearer ${token}` }
});

const { count } = await response.json();
console.log(`Total critical operations: ${count}`);
```

## 🎨 Common Use Cases

### Use Case 1: Audit Log Dashboard

Display recent critical operations:

```javascript
async function fetchRecentCriticalLogs() {
  const response = await fetch(
    '/api/audit-logs/others?severity=CRITICAL&limit=10&skip=0',
    {
      headers: { 'Authorization': `Bearer ${token}` }
    }
  );
  
  const data = await response.json();
  return data.data;
}

// Display in UI
const logs = await fetchRecentCriticalLogs();
logs.forEach(log => {
  console.log(`${log.user.name} performed ${log.action} on ${log.created_at}`);
});
```

### Use Case 2: User Activity History

Show all actions by a specific user:

```javascript
async function fetchUserActivity(userId, page = 1, pageSize = 20) {
  const skip = (page - 1) * pageSize;
  
  const response = await fetch(
    `/api/audit-logs/others?user_id=${userId}&limit=${pageSize}&skip=${skip}`,
    {
      headers: { 'Authorization': `Bearer ${token}` }
    }
  );
  
  return await response.json();
}

// Usage
const userActivity = await fetchUserActivity('507f1f77bcf86cd799439011', 1, 20);
```

### Use Case 3: Grade Change History

Track all grade updates for a submission:

```javascript
async function fetchGradeHistory(submissionId) {
  const response = await fetch(
    `/api/audit-logs/others?entity_id=${submissionId}&action=submission.grade.update`,
    {
      headers: { 'Authorization': `Bearer ${token}` }
    }
  );
  
  const data = await response.json();
  
  // Display grade changes
  data.data.forEach(log => {
    const { before_score, after_score } = log.metadata;
    console.log(
      `${log.user.name} changed grade from ${before_score} to ${after_score}`
    );
  });
}
```

### Use Case 4: Date Range Filter

Get all operations within a date range:

```javascript
async function fetchLogsByDateRange(startDate, endDate) {
  const params = new URLSearchParams({
    from_date: startDate, // "2024-01-01"
    to_date: endDate,     // "2024-12-31"
    limit: '100'
  });
  
  const response = await fetch(`/api/audit-logs/others?${params}`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  
  return await response.json();
}
```

### Use Case 5: Search Audit Logs

Search across multiple fields:

```javascript
async function searchAuditLogs(searchTerm, page = 1) {
  const skip = (page - 1) * 20;
  
  const response = await fetch(
    `/api/audit-logs/others?filter=${encodeURIComponent(searchTerm)}&limit=20&skip=${skip}`,
    {
      headers: { 'Authorization': `Bearer ${token}` }
    }
  );
  
  return await response.json();
}

// Search for "john" - will search in action, entity_type, user_id, etc.
const results = await searchAuditLogs('john');
```

## 📊 Action Types Reference

Common action types you'll encounter:

### Student Operations
- `student.create` - Student created
- `student.update` - Student updated
- `student.delete` - Student deleted

### Teacher Operations
- `teacher.create` - Teacher created
- `teacher.update` - Teacher updated
- `teacher.delete` - Teacher deleted

### Assignment & Grading
- `submission.grade.update` - Grade updated
- `submission.create` - Submission created
- `submission.delete` - Submission deleted
- `assignment.create` - Assignment created
- `assignment.update` - Assignment updated
- `assignment.delete` - Assignment deleted

### Attendance
- `attendance.create` - Attendance recorded
- `attendance.update` - Attendance updated
- `attendance.delete` - Attendance deleted

### Finance
- `finance.transaction.create` - Transaction created
- `finance.transaction.update` - Transaction updated
- `finance.transaction.delete` - Transaction deleted
- `finance.transaction.approve` - Transaction approved

### Roles & Permissions
- `role.assign` - Role assigned to user
- `role.remove` - Role removed from user
- `role.create` - New role created
- `role.update` - Role updated

### System
- `feature.toggle` - Feature enabled/disabled
- `timetable.update` - Timetable modified

## 🎯 Entity Types Reference

- `student` - Student records
- `teacher` - Teacher records
- `submission` - Assignment submissions
- `assignment` - Assignments
- `attendance` - Attendance records
- `transaction` - Financial transactions
- `user` - User accounts
- `role` - Roles
- `timetable` - Timetables
- `feature` - Feature flags

## 🚦 Severity Levels

```typescript
enum AuditSeverity {
  INFO = 'INFO',        // Normal operations (creates, updates, grade changes)
  WARNING = 'WARNING',  // Important changes (role assignments, attendance deletion)
  CRITICAL = 'CRITICAL' // High-impact (finance, deletions, security changes)
}
```

**Use severity for filtering:**
```javascript
// Show only critical operations
const criticalLogs = await fetch('/api/audit-logs?severity=CRITICAL');

// Show warnings and critical
const importantLogs = await fetch('/api/audit-logs?severity=WARNING,CRITICAL');
```

## 🎨 UI Component Examples

### React Component: Audit Log Table

```typescript
import React, { useEffect, useState } from 'react';

interface AuditLog {
  _id: string;
  action: string;
  entity_type: string;
  severity: 'INFO' | 'WARNING' | 'CRITICAL';
  created_at: string;
  user?: { name: string; email: string };
  metadata?: Record<string, any>;
}

const AuditLogTable: React.FC = () => {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    fetchLogs();
  }, [page]);

  const fetchLogs = async () => {
    setLoading(true);
    const skip = (page - 1) * 20;
    
    const response = await fetch(
      `/api/audit-logs/others?limit=20&skip=${skip}`,
      {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      }
    );
    
    const data = await response.json();
    setLogs(data.data);
    setTotal(data.total);
    setLoading(false);
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'CRITICAL': return 'text-red-600 bg-red-100';
      case 'WARNING': return 'text-yellow-600 bg-yellow-100';
      case 'INFO': return 'text-blue-600 bg-blue-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              Date/Time
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              User
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              Action
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              Entity
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
              Severity
            </th>
          </tr>
        </thead>
        <tbody className="bg-white divide-y divide-gray-200">
          {logs.map((log) => (
            <tr key={log._id}>
              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {new Date(log.created_at).toLocaleString()}
              </td>
              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {log.user?.name || 'Unknown'}
              </td>
              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {log.action}
              </td>
              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {log.entity_type}
              </td>
              <td className="px-6 py-4 whitespace-nowrap">
                <span className={`px-2 py-1 text-xs rounded-full ${getSeverityColor(log.severity)}`}>
                  {log.severity}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      
      {/* Pagination */}
      <div className="flex justify-between items-center mt-4">
        <button
          onClick={() => setPage(p => Math.max(1, p - 1))}
          disabled={page === 1}
          className="px-4 py-2 bg-blue-500 text-white rounded disabled:bg-gray-300"
        >
          Previous
        </button>
        <span>Page {page} of {Math.ceil(total / 20)}</span>
        <button
          onClick={() => setPage(p => p + 1)}
          disabled={page >= Math.ceil(total / 20)}
          className="px-4 py-2 bg-blue-500 text-white rounded disabled:bg-gray-300"
        >
          Next
        </button>
      </div>
    </div>
  );
};

export default AuditLogTable;
```

### React Component: Audit Log Filters

```typescript
import React, { useState } from 'react';

interface FilterProps {
  onFilterChange: (filters: AuditFilters) => void;
}

interface AuditFilters {
  severity?: string;
  entity_type?: string;
  action?: string;
  from_date?: string;
  to_date?: string;
}

const AuditLogFilters: React.FC<FilterProps> = ({ onFilterChange }) => {
  const [filters, setFilters] = useState<AuditFilters>({});

  const handleChange = (key: keyof AuditFilters, value: string) => {
    const newFilters = { ...filters, [key]: value || undefined };
    setFilters(newFilters);
    onFilterChange(newFilters);
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Severity
        </label>
        <select
          onChange={(e) => handleChange('severity', e.target.value)}
          className="w-full border border-gray-300 rounded-md px-3 py-2"
        >
          <option value="">All</option>
          <option value="INFO">Info</option>
          <option value="WARNING">Warning</option>
          <option value="CRITICAL">Critical</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Entity Type
        </label>
        <select
          onChange={(e) => handleChange('entity_type', e.target.value)}
          className="w-full border border-gray-300 rounded-md px-3 py-2"
        >
          <option value="">All</option>
          <option value="student">Student</option>
          <option value="teacher">Teacher</option>
          <option value="submission">Submission</option>
          <option value="assignment">Assignment</option>
          <option value="attendance">Attendance</option>
          <option value="transaction">Transaction</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          From Date
        </label>
        <input
          type="date"
          onChange={(e) => handleChange('from_date', e.target.value)}
          className="w-full border border-gray-300 rounded-md px-3 py-2"
        />
      </div>
    </div>
  );
};

export default AuditLogFilters;
```

### Vue.js Component Example

```vue
<template>
  <div class="audit-log-viewer">
    <h2>Audit Logs</h2>
    
    <!-- Filters -->
    <div class="filters">
      <select v-model="filters.severity">
        <option value="">All Severities</option>
        <option value="INFO">Info</option>
        <option value="WARNING">Warning</option>
        <option value="CRITICAL">Critical</option>
      </select>
      
      <input
        v-model="filters.search"
        type="text"
        placeholder="Search..."
        @input="debouncedSearch"
      />
    </div>

    <!-- Table -->
    <table>
      <thead>
        <tr>
          <th>Date</th>
          <th>User</th>
          <th>Action</th>
          <th>Severity</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="log in logs" :key="log._id">
          <td>{{ formatDate(log.created_at) }}</td>
          <td>{{ log.user?.name }}</td>
          <td>{{ log.action }}</td>
          <td>
            <span :class="`badge badge-${log.severity.toLowerCase()}`">
              {{ log.severity }}
            </span>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- Pagination -->
    <div class="pagination">
      <button @click="prevPage" :disabled="page === 1">Previous</button>
      <span>Page {{ page }} of {{ totalPages }}</span>
      <button @click="nextPage" :disabled="page >= totalPages">Next</button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, watch } from 'vue';
import { debounce } from 'lodash';

export default {
  name: 'AuditLogViewer',
  setup() {
    const logs = ref([]);
    const page = ref(1);
    const totalPages = ref(1);
    const filters = ref({
      severity: '',
      search: ''
    });

    const fetchLogs = async () => {
      const params = new URLSearchParams({
        limit: '20',
        skip: String((page.value - 1) * 20),
        ...(filters.value.severity && { severity: filters.value.severity }),
        ...(filters.value.search && { filter: filters.value.search })
      });

      const response = await fetch(`/api/audit-logs/others?${params}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      });

      const data = await response.json();
      logs.value = data.data;
      totalPages.value = data.total_pages;
    };

    const debouncedSearch = debounce(fetchLogs, 500);

    const prevPage = () => {
      if (page.value > 1) {
        page.value--;
        fetchLogs();
      }
    };

    const nextPage = () => {
      if (page.value < totalPages.value) {
        page.value++;
        fetchLogs();
      }
    };

    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleString();
    };

    watch(() => filters.value.severity, fetchLogs);

    onMounted(fetchLogs);

    return {
      logs,
      page,
      totalPages,
      filters,
      prevPage,
      nextPage,
      formatDate,
      debouncedSearch
    };
  }
};
</script>
```

## 🔒 Error Handling

```javascript
async function fetchAuditLogs() {
  try {
    const response = await fetch('/api/audit-logs', {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    });

    if (response.status === 403) {
      // User doesn't have permission
      alert('You do not have permission to view audit logs');
      return;
    }

    if (response.status === 401) {
      // Not authenticated
      window.location.href = '/login';
      return;
    }

    if (!response.ok) {
      throw new Error('Failed to fetch audit logs');
    }

    const data = await response.json();
    return data;
    
  } catch (error) {
    console.error('Error fetching audit logs:', error);
    // Show error to user
  }
}
```

## 📝 Best Practices

1. **Always use pagination** - Don't fetch all logs at once
2. **Filter by date range** - Limit queries to specific time periods
3. **Cache results** - Use React Query or similar for caching
4. **Show loading states** - Audit logs can be large datasets
5. **Format dates properly** - Use user's timezone
6. **Color code severity** - Make critical operations stand out
7. **Add search/filter UI** - Help users find specific logs
8. **Export functionality** - Allow downloading logs as CSV/PDF
9. **Real-time updates** - Consider WebSocket for live updates
10. **Responsive design** - Make tables mobile-friendly

## 🚀 Quick Start Checklist

- [ ] Check user has ADMIN or SCHOOLSTAFF role
- [ ] Include Authorization header in all requests
- [ ] Implement pagination (limit + skip)
- [ ] Add filters for severity, entity_type, action
- [ ] Display user name from relations endpoint
- [ ] Format dates in user's timezone
- [ ] Add error handling for 401/403 responses
- [ ] Show loading states
- [ ] Implement search functionality
- [ ] Add export/download feature

## 📞 Support

If you encounter issues:
1. Check user has correct role (ADMIN or SCHOOLSTAFF)
2. Verify Authorization token is valid
3. Check network tab for API response errors
4. Ensure school_id is properly set in context
5. Contact backend team if issues persist
