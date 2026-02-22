# Learning Materials System - Frontend Implementation Guide

## Overview

The Learning Materials System provides a lightweight LMS layer for managing educational content per subject. Teachers can upload lesson notes, resources, videos, and files, while students and parents can access published materials.

## Architecture

The implementation follows the standard architecture pattern:
- **Domain**: `src/domain/learning_material.rs`
- **Service**: `src/services/learning_material_service.rs`
- **Pipeline**: `src/pipeline/learning_material_pipeline.rs`
- **API**: `src/api/learning_materials_api.rs`

## Data Model

### LearningMaterial Schema

```typescript
interface LearningMaterial {
  _id: string;
  school_id: string;
  class_id: string;
  subject_id: string;
  title: string;
  description?: string;
  material_type: "LESSON_NOTE" | "RESOURCE" | "VIDEO" | "FILE";
  file_url?: string;
  file_public_id?: string;
  video_url?: string;
  uploaded_by: string;
  is_published: boolean;
  deleted_at?: Date;
  created_at: Date;
  updated_at: Date;
}

interface LearningMaterialWithRelations extends LearningMaterial {
  uploader?: User;
  school?: School;
  class?: Class;
  subject?: ClassSubject;
}
```

## API Endpoints

Base URL: `/learning-materials` or `/api/v1/learning-materials`

### 1. Get All Materials

```http
GET /learning-materials
GET /learning-materials?class_id={class_id}&subject_id={subject_id}
GET /learning-materials?material_type=LESSON_NOTE
GET /learning-materials?limit=20&skip=0
```

**Query Parameters:**
- `class_id` - Filter by class
- `subject_id` - Filter by subject
- `material_type` - Filter by type (LESSON_NOTE, RESOURCE, VIDEO, FILE)
- `is_published` - Filter by published status (true/false)
- `limit` - Pagination limit (default: 50)
- `skip` - Pagination offset (default: 0)
- `filter` - Search in title and description

**Response:**
```json
{
  "data": [
    {
      "_id": "...",
      "title": "Introduction to Algebra",
      "description": "Basic algebra concepts",
      "material_type": "LESSON_NOTE",
      "file_url": "https://res.cloudinary.com/...",
      "is_published": true,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 100,
  "total_pages": 5,
  "current_page": 1
}
```

**Access Control:**
- Students and Parents: Only see published materials
- Teachers: See all materials for their classes
- Admin/Staff: See all materials

### 2. Get Materials with Relations

```http
GET /learning-materials/others
```

Returns materials with populated uploader, school, class, and subject information.

**Response:**
```json
{
  "data": [
    {
      "learning_material": { ... },
      "uploader": { "name": "John Doe", ... },
      "class": { "name": "Grade 10A", ... },
      "subject": { "name": "Mathematics", ... }
    }
  ],
  "total": 100,
  "total_pages": 5,
  "current_page": 1
}
```

### 3. Get Single Material

```http
GET /learning-materials/{id}
```

**Response:**
```json
{
  "_id": "...",
  "title": "Introduction to Algebra",
  "material_type": "LESSON_NOTE",
  "file_url": "https://res.cloudinary.com/...",
  "is_published": true
}
```

### 4. Get Single Material with Relations

```http
GET /learning-materials/{id}/others
```

### 5. Create Material

```http
POST /learning-materials
Content-Type: multipart/form-data
```

**Form Data:**
- `data` (JSON string):
  ```json
  {
    "school_id": "...",
    "class_id": "...",
    "subject_id": "...",
    "title": "Introduction to Algebra",
    "description": "Basic algebra concepts",
    "material_type": "LESSON_NOTE",
    "is_published": true
  }
  ```
- `file` (optional): File to upload (PDF, DOCX, PPT, etc.)

**For Video Type:**
```json
{
  "material_type": "VIDEO",
  "video_url": "https://youtube.com/watch?v=..."
}
```

**Response:**
```json
{
  "_id": "...",
  "title": "Introduction to Algebra",
  "file_url": "https://res.cloudinary.com/...",
  "file_public_id": "space-together/school_id/subjects/subject_id/filename"
}
```

**Permissions Required:**
- Admin, Staff, or Teacher role

### 6. Update Material

```http
PUT /learning-materials/{id}
Content-Type: multipart/form-data
```

**Form Data:**
- `data` (JSON string): Partial update fields
- `file` (optional): New file to replace existing

**Note:** If a new file is uploaded, the old file is automatically deleted from Cloudinary.

**Permissions Required:**
- Admin, Staff, or Teacher role

### 7. Delete Material (Soft Delete)

```http
DELETE /learning-materials/{id}
```

Sets `deleted_at` timestamp. Material is hidden but not permanently removed.

**Permissions Required:**
- Admin, Staff, or Teacher role

### 8. Count Materials

```http
GET /learning-materials/count?class_id={class_id}
```

Returns total count matching filters.

**Response:**
```json
{
  "count": 42
}
```

## Frontend Implementation Examples

### React/TypeScript Example

```typescript
// Types
interface LearningMaterial {
  _id: string;
  title: string;
  description?: string;
  material_type: "LESSON_NOTE" | "RESOURCE" | "VIDEO" | "FILE";
  file_url?: string;
  video_url?: string;
  is_published: boolean;
  created_at: string;
}

// Fetch materials for a subject
async function fetchMaterials(subjectId: string, classId: string) {
  const response = await fetch(
    `/api/v1/learning-materials?subject_id=${subjectId}&class_id=${classId}`,
    {
      headers: {
        'Authorization': `Bearer ${token}`,
        'X-School-Token': schoolToken
      }
    }
  );
  return response.json();
}

// Upload material with file
async function uploadMaterial(data: Partial<LearningMaterial>, file?: File) {
  const formData = new FormData();
  formData.append('data', JSON.stringify(data));
  if (file) {
    formData.append('file', file);
  }

  const response = await fetch('/api/v1/learning-materials', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-School-Token': schoolToken
    },
    body: formData
  });
  return response.json();
}

// Update material
async function updateMaterial(id: string, data: Partial<LearningMaterial>, file?: File) {
  const formData = new FormData();
  formData.append('data', JSON.stringify(data));
  if (file) {
    formData.append('file', file);
  }

  const response = await fetch(`/api/v1/learning-materials/${id}`, {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-School-Token': schoolToken
    },
    body: formData
  });
  return response.json();
}

// Delete material
async function deleteMaterial(id: string) {
  const response = await fetch(`/api/v1/learning-materials/${id}`, {
    method: 'DELETE',
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-School-Token': schoolToken
    }
  });
  return response.json();
}
```

### React Component Example

```tsx
import React, { useState, useEffect } from 'react';

function MaterialsList({ subjectId, classId }) {
  const [materials, setMaterials] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchMaterials(subjectId, classId)
      .then(data => {
        setMaterials(data.data);
        setLoading(false);
      });
  }, [subjectId, classId]);

  if (loading) return <div>Loading...</div>;

  return (
    <div className="materials-list">
      {materials.map(material => (
        <div key={material._id} className="material-card">
          <h3>{material.title}</h3>
          <p>{material.description}</p>
          <span className="badge">{material.material_type}</span>
          
          {material.file_url && (
            <a href={material.file_url} target="_blank" rel="noopener noreferrer">
              Download File
            </a>
          )}
          
          {material.video_url && (
            <a href={material.video_url} target="_blank" rel="noopener noreferrer">
              Watch Video
            </a>
          )}
        </div>
      ))}
    </div>
  );
}

function UploadMaterialForm({ subjectId, classId, schoolId }) {
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    material_type: 'FILE',
    is_published: false
  });
  const [file, setFile] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    const data = {
      ...formData,
      school_id: schoolId,
      class_id: classId,
      subject_id: subjectId
    };

    try {
      await uploadMaterial(data, file);
      alert('Material uploaded successfully!');
    } catch (error) {
      alert('Upload failed: ' + error.message);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="Title"
        value={formData.title}
        onChange={e => setFormData({...formData, title: e.target.value})}
        required
      />
      
      <textarea
        placeholder="Description"
        value={formData.description}
        onChange={e => setFormData({...formData, description: e.target.value})}
      />
      
      <select
        value={formData.material_type}
        onChange={e => setFormData({...formData, material_type: e.target.value})}
      >
        <option value="LESSON_NOTE">Lesson Note</option>
        <option value="RESOURCE">Resource</option>
        <option value="VIDEO">Video</option>
        <option value="FILE">File</option>
      </select>
      
      {formData.material_type !== 'VIDEO' && (
        <input
          type="file"
          onChange={e => setFile(e.target.files[0])}
          accept=".pdf,.doc,.docx,.ppt,.pptx,.xls,.xlsx,.zip,.txt"
        />
      )}
      
      {formData.material_type === 'VIDEO' && (
        <input
          type="url"
          placeholder="Video URL"
          value={formData.video_url || ''}
          onChange={e => setFormData({...formData, video_url: e.target.value})}
        />
      )}
      
      <label>
        <input
          type="checkbox"
          checked={formData.is_published}
          onChange={e => setFormData({...formData, is_published: e.target.checked})}
        />
        Publish immediately
      </label>
      
      <button type="submit">Upload Material</button>
    </form>
  );
}
```

## File Storage

### Cloudinary Integration

Files are stored in Cloudinary with the following structure:
```
space-together/{school_id}/subjects/{subject_id}/{filename}
```

**Supported File Types:**
- PDF (.pdf)
- Word Documents (.doc, .docx)
- PowerPoint (.ppt, .pptx)
- Excel (.xls, .xlsx)
- ZIP archives (.zip)
- Text files (.txt)
- Audio files (.mp3, .wav)
- Video files (.mp4) - optional

**File Size Limit:** 50MB per file (configurable)

### File Upload Process

1. Frontend sends multipart form data with `data` (JSON) and `file` (binary)
2. Backend validates file size and type
3. File is uploaded to Cloudinary with resource_type="raw"
4. Cloudinary returns `url` and `public_id`
5. Material record is created with file metadata

### File Deletion

When a material is updated with a new file or deleted:
1. Old `file_public_id` is retrieved
2. `CloudinaryService::delete_file()` is called
3. File is removed from Cloudinary
4. New file is uploaded (if updating)

## Permission System

### Role-Based Access

| Role | Create | Read | Update | Delete |
|------|--------|------|--------|--------|
| Admin | ✅ | ✅ All | ✅ | ✅ |
| Staff | ✅ | ✅ All | ✅ | ✅ |
| Teacher | ✅ | ✅ Class-scoped | ✅ | ✅ |
| Student | ❌ | ✅ Published only | ❌ | ❌ |
| Parent | ❌ | ✅ Published only | ❌ | ❌ |

### Permission Checks

Teachers must belong to the class to create/update materials:
```rust
check_admin_staff_or_teacher(&user)
```

Students and parents automatically filtered to published materials only:
```rust
if matches!(user.role, Some(UserRole::STUDENT) | Some(UserRole::PARENT)) {
    extra_match.insert("is_published", true);
}
```

## Audit Logging

All operations are logged:
- `learning_material.create` - Info severity
- `learning_material.update` - Info severity
- `learning_material.delete` - Warning severity

Audit logs include:
- User who performed action
- School ID
- Entity ID
- Timestamp
- User role

## Filtering and Search

### Available Filters

```http
GET /learning-materials?class_id={id}&subject_id={id}&material_type=LESSON_NOTE&is_published=true
```

### Search

```http
GET /learning-materials?filter=algebra
```

Searches in: `title`, `description`

### Pagination

```http
GET /learning-materials?limit=20&skip=40
```

Returns page 3 (items 41-60)

## Best Practices

### For Teachers

1. **Organize by Subject**: Upload materials to the correct subject
2. **Use Descriptive Titles**: Help students find materials easily
3. **Publish When Ready**: Use `is_published: false` for drafts
4. **Choose Correct Type**: 
   - LESSON_NOTE: Class notes, lecture slides
   - RESOURCE: Additional reading, worksheets
   - VIDEO: YouTube links, recorded lectures
   - FILE: General documents

### For Frontend Developers

1. **Handle File Uploads**: Use FormData for multipart requests
2. **Show Upload Progress**: Large files may take time
3. **Validate File Types**: Check before upload
4. **Display Material Type**: Use icons or badges
5. **Filter by Subject**: Show materials relevant to current context
6. **Respect Permissions**: Hide create/edit buttons for students
7. **Handle Errors**: Show user-friendly messages

### Security Considerations

1. **Authentication Required**: All endpoints require JWT token
2. **School Token**: Multi-tenant isolation via X-School-Token header
3. **File Validation**: Backend validates file size and type
4. **Soft Delete**: Materials are never permanently deleted
5. **Audit Trail**: All actions are logged

## Error Handling

### Common Errors

```json
{
  "message": "Title is required"
}
```

```json
{
  "message": "School ID is required"
}
```

```json
{
  "message": "Video URL is required for VIDEO type"
}
```

```json
{
  "message": "File size exceeded 50MB"
}
```

```json
{
  "message": "Access denied"
}
```

### HTTP Status Codes

- `200 OK` - Success
- `201 Created` - Material created
- `400 Bad Request` - Validation error
- `403 Forbidden` - Permission denied
- `404 Not Found` - Material not found
- `500 Internal Server Error` - Server error

## Testing

### Manual Testing

1. **Create Material**:
   ```bash
   curl -X POST http://localhost:8080/api/v1/learning-materials \
     -H "Authorization: Bearer $TOKEN" \
     -H "X-School-Token: $SCHOOL_TOKEN" \
     -F 'data={"title":"Test","school_id":"...","class_id":"...","subject_id":"...","material_type":"FILE","is_published":true}' \
     -F 'file=@test.pdf'
   ```

2. **Get Materials**:
   ```bash
   curl http://localhost:8080/api/v1/learning-materials?subject_id=... \
     -H "Authorization: Bearer $TOKEN" \
     -H "X-School-Token: $SCHOOL_TOKEN"
   ```

3. **Update Material**:
   ```bash
   curl -X PUT http://localhost:8080/api/v1/learning-materials/{id} \
     -H "Authorization: Bearer $TOKEN" \
     -H "X-School-Token: $SCHOOL_TOKEN" \
     -F 'data={"title":"Updated Title"}' \
     -F 'file=@new_file.pdf'
   ```

4. **Delete Material**:
   ```bash
   curl -X DELETE http://localhost:8080/api/v1/learning-materials/{id} \
     -H "Authorization: Bearer $TOKEN" \
     -H "X-School-Token: $SCHOOL_TOKEN"
   ```

## Database Indexes

The following indexes are automatically created:
- `school_id`
- `class_id`
- `subject_id`
- `uploaded_by`
- `material_type`
- `is_published`
- `created_at`
- `deleted_at`
- Compound: `(school_id, subject_id, created_at)`
- Compound: `(school_id, class_id, created_at)`

## Summary

The Learning Materials System provides:
- ✅ File upload to Cloudinary (PDF, DOCX, PPT, etc.)
- ✅ Video link support
- ✅ Per-subject organization
- ✅ Role-based permissions
- ✅ Published/draft status
- ✅ Soft delete
- ✅ Audit logging
- ✅ Filtering and search
- ✅ Pagination
- ✅ Multi-tenant isolation

All endpoints follow REST conventions and integrate with the existing permission and audit systems.
