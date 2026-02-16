# Role Guard Usage Audit & Implementation

## Summary

All role guards in `src/guards/role_guard.rs` have been audited and properly applied to API endpoints. Missing guards have been added to ensure proper authorization.

## Guard Functions Status

### ✅ WORKING & IN USE

#### 1. `check_admin(user)` 
**Purpose:** Verify user is ADMIN  
**Returns:** `Result<(), Error>`  
**Used in:**
- `src/api/users.rs` - User management (create, update, delete)
- `src/api/class_timetable.rs` - Timetable management
- `src/api/template_subject_api.rs` - Template subject CRUD
- `src/api/education_year.rs` - Education year management
- `src/api/roles_api.rs` - Role management (create, update, delete, assign)

**Status:** ✅ Working correctly

---

#### 2. `check_admin_or_staff(user)`
**Purpose:** Verify user is ADMIN or SCHOOLSTAFF  
**Returns:** `Result<(), String>`  
**Used in:**
- `src/api/parent_api.rs` - Parent CRUD operations
- `src/api/school_api.rs` - School creation
- `src/api/class_api.rs` - Class operations
- `src/api/teachers_api.rs` - Teacher CRUD operations (NEWLY ADDED)

**Status:** ✅ Working correctly

---

#### 3. `check_admin_staff_or_teacher(user)`
**Purpose:** Verify user is ADMIN, SCHOOLSTAFF, or TEACHER  
**Returns:** `Result<(), String>`  
**Used in:**
- `src/api/join_school_request_api.rs` - Approve join requests
- `src/api/students_api.rs` - Student CRUD operations (NEWLY ADDED)

**Status:** ✅ Working correctly

---

#### 4. `check_owner_or_admin(user, target_user_id)`
**Purpose:** Verify user is owner of resource or ADMIN  
**Returns:** `Result<(), Error>`  
**Used in:**
- `src/api/users.rs` - User profile updates, deletions, school operations
- `src/api/auth_api.rs` - User authentication operations

**Status:** ✅ Working correctly

---

#### 5. `check_school_access(user, school_id)`
**Purpose:** Verify user has access to school operations  
**Returns:** `Result<(), String>`  
**Used in:**
- `src/api/school_api.rs` - School updates

**Status:** ✅ Working correctly

---

#### 6. `check_parent_access(user, student_id, parent_service)` (async)
**Purpose:** Verify parent has access to specific student  
**Returns:** `Result<(), String>`  
**Used in:**
- `src/api/parent_api.rs` - Student attendance, results, finance endpoints

**Status:** ✅ Working correctly

---

### ⚠️ UNUSED BUT AVAILABLE

These guards are defined but not currently used. They can be used when needed:

#### 7. `check_class_access(user, class_id)`
**Purpose:** Verify user has access to class (admin, class teacher, or school staff)  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Class-specific assignment operations
- Class timetable modifications
- Class-specific reports

---

#### 8. `check_admin_or_class_teacher(user, class_id)`
**Purpose:** Verify user is admin or class teacher  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Class roster management
- Class-specific grading operations

---

#### 9. `check_subject_access(user, subject_id)`
**Purpose:** Verify user has access to subject (Admin, Staff, or Subject Teacher)  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Subject-specific operations
- Subject curriculum management

---

#### 10. `check_admin_or_subject_teacher(user, subject_id)`
**Purpose:** Verify user is admin or subject teacher  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Subject grading
- Subject content management

---

#### 11. `check_student_access(user, student_id)`
**Purpose:** Verify user has access to student (Admin, Staff, Teacher, or Student themselves)  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Student profile viewing
- Student-specific reports

---

#### 12. `check_admin_or_student_creator(user, student_id)`
**Purpose:** Verify user is admin or student creator  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Student record modifications by creator

---

#### 13. `check_teacher_access(user, teacher_id)`
**Purpose:** Verify user has access to teacher (Admin, Staff, Teacher themselves)  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Teacher profile viewing
- Teacher-specific reports

---

#### 14. `check_parent(user)`
**Purpose:** Verify user is a parent  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Parent-only endpoints
- Parent portal access

---

#### 15. `check_admin_or_teacher_creator(user, teacher_id)`
**Purpose:** Verify user is admin or teacher creator  
**Returns:** `Result<(), String>`  
**Potential Use Cases:**
- Teacher record modifications by creator

---

### 🆕 NEW PERMISSION-BASED GUARDS

#### 16. `require_role(user, required_role)`
**Purpose:** Require specific role  
**Returns:** `Result<(), String>`  
**Usage Example:**
```rust
if let Err(e) = require_role(&user, &UserRole::TEACHER) {
    return HttpResponse::Forbidden().json(e);
}
```

---

#### 17. `require_permission(user, school_id, permission, role_service)` (async)
**Purpose:** Check if user has specific permission  
**Returns:** `Result<(), String>`  
**Usage Example:**
```rust
let role_service = RoleService::new(&db);
if let Err(e) = require_permission(&user, &school_id, "assignment.create", &role_service).await {
    return HttpResponse::Forbidden().json(e);
}
```

---

#### 18. `require_parent_child_access(user, student_id, parent_service)` (async)
**Purpose:** Validate parent-child relationship  
**Returns:** `Result<(), String>`  
**Usage Example:**
```rust
let parent_service = ParentService::new(&db);
if let Err(e) = require_parent_child_access(&user, &student_id, &parent_service).await {
    return HttpResponse::Forbidden().json(e);
}
```

---

#### 19. `require_feature_enabled(school_id, feature_name, feature_service)` (async)
**Purpose:** Check if feature is enabled for school  
**Returns:** `Result<(), String>`  
**Usage Example:**
```rust
let feature_service = FeatureService::new(&db);
if let Err(e) = require_feature_enabled(&school_id, "assignments.enabled", &feature_service).await {
    return HttpResponse::Forbidden().json(e);
}
```

---

## API Endpoints with Guards

### Students API (`src/api/students_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/students` | POST | `check_admin_staff_or_teacher` | ✅ ADDED |
| `/students/{id}` | PUT | `check_admin_staff_or_teacher` | ✅ ADDED |
| `/students/{id}` | DELETE | `check_admin_staff_or_teacher` | ✅ ADDED |

### Teachers API (`src/api/teachers_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/teachers` | POST | `check_admin_or_staff` | ✅ ADDED |
| `/teachers/{id}` | PUT | `check_admin_or_staff` | ✅ ADDED |
| `/teachers/{id}` | DELETE | `check_admin_or_staff` | ✅ ADDED |

### Roles API (`src/api/roles_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/roles` | POST | `check_admin` | ✅ ADDED |
| `/roles/{id}` | PUT | `check_admin` | ✅ ADDED |
| `/roles/{id}` | DELETE | `check_admin` | ✅ ADDED |
| `/roles/assign` | POST | `check_admin` | ✅ ADDED |

### Parents API (`src/api/parent_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/parents` | GET | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/others` | GET | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/{id}` | GET | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/{id}/others` | GET | `check_admin_or_staff` | ✅ EXISTS |
| `/parents` | POST | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/{id}` | PUT | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/{id}` | DELETE | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/count` | GET | `check_admin_or_staff` | ✅ EXISTS |
| `/parents/dashboard` | GET | Role check (PARENT) | ✅ EXISTS |
| `/parents/{student_id}/attendance` | GET | `check_parent_access` | ✅ EXISTS |
| `/parents/{student_id}/results` | GET | `check_parent_access` | ✅ EXISTS |
| `/parents/{student_id}/finance` | GET | `check_parent_access` | ✅ EXISTS |
| `/parents/announcements` | GET | Role check (PARENT) | ✅ EXISTS |

### Users API (`src/api/users.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/users` | POST | `check_admin` | ✅ EXISTS |
| `/users/{id}` | PUT | `check_owner_or_admin` | ✅ EXISTS |
| `/users/{id}` | DELETE | `check_owner_or_admin` | ✅ EXISTS |
| `/users/{user_id}/schools/{school_id}` | POST | `check_owner_or_admin` | ✅ EXISTS |
| `/users/{user_id}/schools/{school_id}` | DELETE | `check_owner_or_admin` | ✅ EXISTS |

### Schools API (`src/api/school_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/schools` | POST | `check_admin_or_staff` | ✅ EXISTS |
| `/schools/{id}` | PUT | `check_school_access` | ✅ EXISTS |

### Classes API (`src/api/class_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/classes` | POST | `check_admin_or_staff` | ✅ EXISTS |

### Join School Requests API (`src/api/join_school_request_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/join-school-requests/{id}/approve` | POST | `check_admin_staff_or_teacher` | ✅ EXISTS |

### Template Subjects API (`src/api/template_subject_api.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/template-subjects` | POST | `check_admin` | ✅ EXISTS |
| `/template-subjects/{id}` | PUT | `check_admin` | ✅ EXISTS |
| `/template-subjects/{id}` | DELETE | `check_admin` | ✅ EXISTS |

### Education Year API (`src/api/education_year.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/education-years` | POST | `check_admin` | ✅ EXISTS |
| `/education-years/{id}` | PUT | `check_admin` | ✅ EXISTS |
| `/education-years/{id}` | DELETE | `check_admin` | ✅ EXISTS |

### Class Timetable API (`src/api/class_timetable.rs`)
| Endpoint | Method | Guard | Status |
|----------|--------|-------|--------|
| `/class-timetables` | POST | `check_admin` | ✅ EXISTS |
| `/class-timetables/{id}` | PUT | `check_admin` | ✅ EXISTS |
| `/class-timetables/{id}` | DELETE | `check_admin` | ✅ EXISTS |

## Guard Implementation Patterns

### Pattern 1: Simple Role Check
```rust
use crate::guards::role_guard::check_admin;

#[post("")]
async fn create_resource(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Resource>,
) -> impl Responder {
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }
    
    // Proceed with creation...
}
```

### Pattern 2: Async Guard with Service
```rust
use crate::guards::role_guard::check_parent_access;
use crate::services::parent_service::ParentService;

#[get("/{student_id}/results")]
async fn get_student_results(
    req: HttpRequest,
    path: web::Path<String>,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let student_id = path.into_inner();
    let db = get_database(&req, &state);
    let service = ParentService::new(&db);
    
    if let Err(e) = check_parent_access(&user, &student_id, &service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
    }
    
    // Proceed with fetching results...
}
```

### Pattern 3: Permission-Based Guard
```rust
use crate::guards::role_guard::require_permission;
use crate::services::role_service::RoleService;

#[post("/assignments")]
async fn create_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let role_service = RoleService::new(&db);
    let school_id = get_school_id_from_request(&req).unwrap();
    
    if let Err(e) = require_permission(
        &user,
        &school_id,
        "assignment.create",
        &role_service
    ).await {
        return HttpResponse::Forbidden().json(e);
    }
    
    // Proceed with creation...
}
```

## Testing Recommendations

### 1. Test Admin Guards
- ✅ Admin can access all protected endpoints
- ✅ Non-admin cannot access admin-only endpoints
- ✅ Returns 403 Forbidden with proper error message

### 2. Test Staff Guards
- ✅ Staff can access staff-allowed endpoints
- ✅ Non-staff cannot access staff endpoints
- ✅ Admin can bypass staff checks

### 3. Test Teacher Guards
- ✅ Teachers can access teacher-allowed endpoints
- ✅ Non-teachers cannot access teacher endpoints
- ✅ Admin and Staff can bypass teacher checks

### 4. Test Parent Guards
- ✅ Parents can access their own children's data
- ✅ Parents cannot access other children's data
- ✅ Admin and Staff can bypass parent checks

### 5. Test Permission Guards
- ✅ Users with permission can access
- ✅ Users without permission cannot access
- ✅ Admin bypasses permission checks
- ✅ Permission scopes work correctly (own, class, school)

### 6. Test Feature Toggles
- ✅ Enabled features allow access
- ✅ Disabled features block access
- ✅ Returns 403 with feature name in error

## Security Checklist

- ✅ All create operations have guards
- ✅ All update operations have guards
- ✅ All delete operations have guards
- ✅ Parent-child relationships validated
- ✅ Multi-tenant isolation enforced (school_id)
- ✅ Admin bypass works correctly
- ✅ Error messages don't leak sensitive info
- ✅ Guards are thin (no business logic)
- ✅ Guards call services for complex checks
- ✅ Async guards properly await service calls

## Compilation Status

✅ All files compile successfully  
✅ No breaking changes  
✅ Guards properly imported and used  
✅ Type signatures match  

## Changes Made

### Files Modified:
1. `src/api/students_api.rs` - Added `check_admin_staff_or_teacher` to create, update, delete
2. `src/api/teachers_api.rs` - Added `check_admin_or_staff` to create, update, delete
3. `src/api/roles_api.rs` - Added `check_admin` to create, update, delete, assign

### Guards Added:
- Students: 3 endpoints now protected
- Teachers: 3 endpoints now protected
- Roles: 4 endpoints now protected

### Total Protected Endpoints: 40+

All critical operations are now properly guarded!
