# Role Guard + Permission Granularity

Before writing any code:

1. Read how student module is structured:

   * student.rs
   * student_service.rs
   * student_pipeline.rs
   * students_api.rs

2. Follow the exact same naming convention and separation.

3. Do NOT introduce `/api/v1/*`.

4. All endpoints must use `/`.

5. Respect multi-tenant isolation using school_id.

6. Do NOT place business logic inside guards.

7. Do NOT break existing UserRole enum.

8. Extend system without rewriting it.

Now implement:

==================================================
PART 1 – Permission Naming Standard
===================================

Use naming format:

<domain>.<resource>.<action>

Examples:
assignment.create
submission.grade
parent.read.child.assignment
role.assign
feature.toggle

Seed default permissions automatically.

==================================================
PART 2 – Role Guard Implementation
==================================

Update role_guard.rs to include:

1. require_role(...)
2. require_permission(...)
3. require_parent_child_access(...)

Guards must:

* Be thin
* Call services
* Not query database directly

==================================================
PART 3 – Role Service
=====================

Create:

role.rs
role_service.rs
role_pipeline.rs
roles_api.rs

Follow exact pattern used in student module.

role_service must include:

* create_role
* update_role
* delete_role
* assign_role_to_user
* user_has_permission(user_id, school_id, permission)

==================================================
PART 4 – Permission Granularity Extensions
==========================================

Support:

1. Own scope:
   submission.read.own

2. Class scope:
   submission.read.class

3. School scope:
   submission.read.school

Service must evaluate:

* If own → match user_id
* If class → verify teacher assigned to class
* If school → allow admin

==================================================
PART 5 – Parent Role
====================

Add Parent to UserRole enum.

Create:

parent_service.rs function:

is_parent_of(parent_id, student_id)

Guard must validate parent-child relationship before access.

==================================================
PART 6 – Feature Toggle Guard
=============================

Add:

require_feature_enabled(school_id, "assignments.enabled")

Check in FeatureService.
Return 403 if disabled.

==================================================
PART 7 – Security Rules
=======================

* Always filter by school_id.
* Prevent cross-school role assignment.
* Prevent removing last Admin role.
* Prevent deletion of system roles.
* Validate permission existence before attaching.

==================================================
GOAL
====

After implementation:

* Roles work correctly.
* Parents are supported.
* Permission system is granular.
* Feature toggles are per school.
* Architecture matches student module.
* API style uses `/` only.
* System is backward compatible.

Do NOT invent new structure.
Follow Domain → Service → Pipeline → API pattern exactly like student module.

---

If you want next, I can design:

* A hierarchical permission resolver optimized for 50k+ users
* Or a permission caching strategy using request-scoped context
* Or a visual permission matrix for Space-Together internal documentation
