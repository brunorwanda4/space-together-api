Lightweight LMS Layer (STRICT ARCHITECTURE)

Before writing any code:

1. Study how modules are structured:

   * `student.rs`
   * `student_service.rs`
   * `student_pipeline.rs`
   * `students_api.rs`
2. Follow exactly the same structure.
3. Do NOT change architecture.
6. Integrate with:

   * role_guard
   * permission system
   * audit log system
7. Use Cloudinary for file storage.
8. Extend `cloudinary_service.rs` to support non-image files.
9. If you need database query [create, update, delete, get] and others use base_repo.rs if that query not exit create it in base_repo.rs and use it
10. after finish make doc file which explain what you did for to implement on fortend

---

# 🎯 GOAL

Implement lightweight LMS features:

Per Subject:

* Lesson notes upload
* Class resources
* Video links
* File repository
* Organized per subject
* Filterable
* Permission protected

---

# 1️⃣ DOMAIN STRUCTURE

Create module:

```
learning_material.rs
learning_material_service.rs
learning_material_pipeline.rs
learning_materials_api.rs
```

---

## LearningMaterial Schema

```rust
pub struct LearningMaterial {
    pub _id: ObjectId,
    pub school_id: ObjectId,
    pub class_id: ObjectId,
    pub subject_id: ObjectId,

    pub title: String,
    pub description: Option<String>,

    pub material_type: String, 
    // "LESSON_NOTE" | "RESOURCE" | "VIDEO" | "FILE"

    pub file_url: Option<String>,
    pub file_public_id: Option<String>,
    pub video_url: Option<String>,

    pub uploaded_by: ObjectId,
    pub is_published: bool,

    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

# 2️⃣ PERMISSION RULES

Teachers:

* learning_material.create.class
* learning_material.update.class
* learning_material.delete.class
* learning_material.read.class

Students:

* learning_material.read.class (published only)

Parents:

* learning_material.read.class (published only)

Admin:

* full access

---

# 3️⃣ CLOUDINARY EXTENSION (IMPORTANT)

Modify:

`cloudinary_service.rs`

Currently supports:

* image upload only

Extend to support:

### 1. Raw file uploads

Use Cloudinary resource_type:

```
resource_type = "raw"
```

Support:

* PDF
* DOCX
* PPT
* XLSX
* ZIP
* TXT
* MP4 (optional)
* Audio files

---

### Add new functions:

```rust
pub async fn upload_file(
    file_bytes: Vec<u8>,
    file_name: &str,
    folder: &str
) -> Result<CloudinaryUploadResult, Error>
```

Return:

```
{
   url,
   public_id,
   format,
   bytes
}
```

Folder format:

```
space-together/{school_id}/subjects/{subject_id}
```

---

### Delete file

Add:

```rust
pub async fn delete_file(public_id: &str)
```

Used when deleting material.

---

# 4️⃣ SERVICE LOGIC

Implement in service layer:

### Create material

* Validate teacher belongs to class
* Upload file if file provided
* Store video_url if type == VIDEO
* Store metadata
* Log audit:
  "learning_material.create"

---

### Update material

* If file replaced:

  * delete old file from Cloudinary
  * upload new file
* Log audit:
  "learning_material.update"

---

### Soft delete

Set deleted_at
Log audit

---

# 5️⃣ PIPELINE

Implement:

GET `/learning-materials`

Query params:

```
class_id
subject_id
material_type
limit
skip
```

Filter:

* If student → only published
* If teacher → class scoped
* Always filter deleted_at = null

---

# 6️⃣ API ROUTES

Create:

```
POST   /learning-materials
GET    /learning-materials
GET    /learning-materials/{id}
PUT    /learning-materials/{id}
DELETE /learning-materials/{id}
```

No `/api/v1`.

---

# 7️⃣ FILE SIZE LIMIT

Add validation:

* Max 50MB per file (configurable)
* Restrict unsafe file types

---

# 8️⃣ INDEXING

Indexes:

```
school_id
class_id
subject_id
material_type
created_at
deleted_at
```

Compound:

```
{ school_id: 1, subject_id: 1, created_at: -1 }
```

---

# 🔐 BACKEND RESULT

After implementation:

* Each subject has repository
* Teachers upload files
* Students access published materials
* Cloudinary handles all files
* Soft delete works
* Audit logging integrated
* Permission system enforced
* Tenant isolated

Follow Domain → Service → Pipeline → API strictly.
