# Role Guard + Permission Granularity Implementation

## Overview

This implementation adds a comprehensive role-based permission system with granular access control, parent-child relationship validation, and feature toggles. The system follows the exact same architectural pattern as the student module.

## Architecture

### Domain Layer (`src/domain/role.rs`)
- `Role`: Main role entity with permissions array
- `RoleType`: System vs Custom roles
- `PermissionScope`: Own, Class, School scopes
- `Permission`: Permission definition with scope
- `UserRoleAssignment`: Links users to roles per school
- `RoleWithRelations`: Role with populated school data

### Service Layer (`src/services/role_service.rs`)
Core business logic:
- `create_role`: Create new roles with validation
- `update_role`: Update roles (prevents system role modification)
- `delete_role`: Delete roles (prevents system role deletion, checks assignments)
- `assign_role_to_user`: Assign roles to users with cross-school validation
- `user_has_permission`: Check if user has specific permission
- `get_default_permissions`: Returns list of default permissions

Security rules implemented:
- ✅ Always filter by school_id
- ✅ Prevent cross-school role assignment
- ✅ Prevent removing last Admin role (via assignment count check)
- ✅ Prevent deletion of system roles
- ✅ Validate permission existence before attaching

### Pipeline Layer (`src/pipeline/role_pipeline.rs`)
MongoDB aggregation pipeline for:
- Role lookups with school relations
- Proper ObjectId normalization
- Sorted results

### API Layer (`src/api/roles_api.rs`)
REST endpoints following `/` pattern (no `/api/v1`):
- `GET /roles` - List all roles
- `GET /roles/others` - List roles with relations
- `GET /roles/{id}` - Get role by ID
- `GET /roles/{id}/others` - Get role with relations
- `GET /roles/match` - Find role by query
- `GET /roles/count` - Count roles
- `GET /roles/permissions` - Get default permissions list
- `POST /roles` - Create role (JWT protected)
- `PUT /roles/{id}` - Update role (JWT protected)
- `DELETE /roles/{id}` - Delete role (JWT protected)
- `POST /roles/assign` - Assign role to user (JWT protected)

## Permission System

### Permission Naming Convention
Format: `<domain>.<resource>.<action>[.<scope>]`

Examples:
```
assignment.create
assignment.read.own
assignment.read.class
assignment.read.school
assignment.update
assignment.delete
submission.grade
submission.read.own
submission.read.class
submission.read.school
parent.read.child.assignment
parent.read.child.submission
role.assign
role.create
role.update
role.delete
feature.toggle
```

### Permission Scopes

1. **Own Scope** (`PermissionScope::Own`)
   - User can only access their own resources
   - Example: `submission.read.own` - student reads their own submissions

2. **Class Scope** (`PermissionScope::Class`)
   - User can access resources within their assigned classes
   - Example: `submission.read.class` - teacher reads class submissions
   - Validates teacher is assigned to the class

3. **School Scope** (`PermissionScope::School`)
   - User can access all resources within the school
   - Example: `submission.read.school` - admin reads all submissions

## Guard Functions

### Updated `src/guards/role_guard.rs`

#### New Functions

1. **`require_role(user, required_role)`**
   - Checks if user has specific role
   - Thin guard, no database queries

2. **`require_permission(user, school_id, permission, role_service)`** (async)
   - Checks if user has specific permission
   - Calls `role_service.user_has_permission()`
   - Admin bypass built-in

3. **`require_parent_child_access(user, student_id, parent_service)`** (async)
   - Validates parent-child relationship
   - Admin/Staff bypass
   - Calls `parent_service.validate_parent_student_access()`

4. **`require_feature_enabled(school_id, feature_name, feature_service)`** (async)
   - Checks if feature is enabled for school
   - Returns 403 if disabled

### Existing Functions (Preserved)
All existing guard functions remain unchanged:
- `is_admin`
- `is_owner_or_admin`
- `check_owner_or_admin`
- `check_admin`
- `check_admin_or_staff`
- `check_school_access`
- `check_admin_staff_or_teacher`
- `check_parent_access` (async)
- And all other existing guards

## Parent Role Support

### UserRole Enum
The `UserRole::PARENT` variant already exists in `src/domain/common_details.rs`:
```rust
pub enum UserRole {
    STUDENT,
    TEACHER,
    ADMIN,
    SCHOOLSTAFF,
    PARENT,
}
```

### Parent Service Enhancement
Added to `src/services/parent_service.rs`:
```rust
pub async fn is_parent_of(
    &self,
    parent_id: &IdType,
    student_id: &IdType,
) -> Result<bool, AppError>
```

This function:
- Validates parent-child relationship via `student_ids` array
- Used by `require_parent_child_access` guard
- Respects multi-tenant isolation (school_id)

## Feature Toggle System

### Feature Service (`src/services/feature_service.rs`)

```rust
pub struct FeatureService {
    pub collection: Collection<SchoolFeatures>,
}
```

Methods:
- `is_feature_enabled(school_id, feature_name)` - Check if feature is enabled
- `toggle_feature(school_id, feature_name, enabled)` - Enable/disable feature

Features are stored per school in `school_features` collection:
```json
{
  "_id": ObjectId("school_id"),
  "features": {
    "assignments.enabled": true,
    "parent_portal.enabled": false,
    "advanced_analytics.enabled": true
  }
}
```

Default behavior: Features are enabled unless explicitly disabled.

## Usage Examples

### Example 1: Protect Assignment Creation
```rust
use crate::guards::role_guard::require_permission;
use crate::services::role_service::RoleService;

#[post("/assignments")]
async fn create_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Assignment>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let role_service = RoleService::new(&db);
    
    // Check permission
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

### Example 2: Parent Accessing Child's Assignment
```rust
use crate::guards::role_guard::require_parent_child_access;
use crate::services::parent_service::ParentService;

#[get("/students/{student_id}/assignments")]
async fn get_student_assignments(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let parent_service = ParentService::new(&db);
    let student_id = path.into_inner();
    
    // Validate parent-child access
    if let Err(e) = require_parent_child_access(
        &user,
        &student_id,
        &parent_service
    ).await {
        return HttpResponse::Forbidden().json(e);
    }
    
    // Proceed with fetching assignments...
}
```

### Example 3: Feature Toggle Check
```rust
use crate::guards::role_guard::require_feature_enabled;
use crate::services::feature_service::FeatureService;

#[post("/assignments")]
async fn create_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let feature_service = FeatureService::new(&db);
    
    // Check if assignments feature is enabled
    if let Err(e) = require_feature_enabled(
        &school_id,
        "assignments.enabled",
        &feature_service
    ).await {
        return HttpResponse::Forbidden().json(e);
    }
    
    // Proceed...
}
```

### Example 4: Scope-Based Permission Evaluation

The service evaluates scopes automatically:

```rust
// Own scope: Match user_id
if permission.ends_with(".own") {
    // Check if resource.user_id == user.id
}

// Class scope: Verify teacher assigned to class
if permission.ends_with(".class") {
    // Check if user.accessible_classes.contains(resource.class_id)
}

// School scope: Allow admin
if permission.ends_with(".school") {
    // Admin has full access
}
```

## Database Collections

### `roles`
```json
{
  "_id": ObjectId,
  "school_id": ObjectId,
  "name": "Teacher Assistant",
  "description": "Can grade assignments and view student work",
  "role_type": "Custom",
  "permissions": [
    "submission.grade",
    "submission.read.class",
    "assignment.read.class"
  ],
  "is_active": true,
  "created_at": ISODate,
  "updated_at": ISODate
}
```

### `user_role_assignments`
```json
{
  "_id": ObjectId,
  "user_id": ObjectId,
  "role_id": ObjectId,
  "school_id": ObjectId,
  "assigned_at": ISODate
}
```

### `school_features`
```json
{
  "_id": ObjectId, // school_id
  "features": {
    "assignments.enabled": true,
    "parent_portal.enabled": true,
    "advanced_analytics.enabled": false
  }
}
```

## Multi-Tenant Isolation

All operations respect school_id:
- ✅ Roles are scoped to schools
- ✅ Role assignments are scoped to schools
- ✅ Permission checks validate school context
- ✅ Cross-school role assignment is prevented
- ✅ Feature toggles are per-school

## Backward Compatibility

- ✅ All existing guard functions preserved
- ✅ UserRole enum not modified (PARENT already existed)
- ✅ No breaking changes to existing APIs
- ✅ System extends without rewriting

## Module Registration

All modules properly registered:
- ✅ `src/domain/mod.rs` - Added `pub mod role;`
- ✅ `src/services/mod.rs` - Added `pub mod role_service;` and `pub mod feature_service;`
- ✅ `src/pipeline/mod.rs` - Added `pub mod role_pipeline;`
- ✅ `src/api/mod.rs` - Added `mod roles_api;` and `roles_api::init(cfg);`

## Testing Checklist

### Role Management
- [ ] Create system role
- [ ] Create custom role
- [ ] Update custom role
- [ ] Attempt to update system role (should fail)
- [ ] Delete custom role without assignments
- [ ] Attempt to delete role with assignments (should fail)
- [ ] Attempt to delete system role (should fail)

### Role Assignment
- [ ] Assign role to user
- [ ] Attempt to assign role from different school (should fail)
- [ ] Attempt to assign inactive role (should fail)
- [ ] Verify user has permission after assignment

### Permission Checks
- [ ] Admin bypasses permission checks
- [ ] User with permission passes check
- [ ] User without permission fails check
- [ ] Permission check with own scope
- [ ] Permission check with class scope
- [ ] Permission check with school scope

### Parent Access
- [ ] Parent accesses own child's data
- [ ] Parent attempts to access non-child data (should fail)
- [ ] Admin bypasses parent check
- [ ] Staff bypasses parent check

### Feature Toggles
- [ ] Enable feature for school
- [ ] Disable feature for school
- [ ] Access with enabled feature
- [ ] Access with disabled feature (should fail)

## Next Steps (Optional Enhancements)

1. **Hierarchical Permission Resolver**
   - Optimize for 50k+ users
   - Cache permission lookups
   - Use Redis for permission cache

2. **Permission Caching Strategy**
   - Request-scoped context
   - Reduce database queries
   - Invalidate on role changes

3. **Visual Permission Matrix**
   - Internal documentation
   - Role comparison view
   - Permission audit trail

4. **Audit Logging**
   - Track role assignments
   - Track permission checks
   - Track feature toggle changes

## Files Created/Modified

### Created
- `src/domain/role.rs`
- `src/services/role_service.rs`
- `src/services/feature_service.rs`
- `src/pipeline/role_pipeline.rs`
- `src/api/roles_api.rs`

### Modified
- `src/guards/role_guard.rs` - Added 4 new guard functions
- `src/services/parent_service.rs` - Added `is_parent_of` method
- `src/domain/mod.rs` - Registered role module
- `src/services/mod.rs` - Registered role_service and feature_service
- `src/pipeline/mod.rs` - Registered role_pipeline
- `src/api/mod.rs` - Registered roles_api

## Compilation Status

✅ All files compile successfully
✅ No breaking changes
✅ Only dead code warnings (unused structs in other modules)
