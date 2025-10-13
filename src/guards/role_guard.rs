use crate::domain::auth_user::AuthUserDto;
use crate::domain::user_role::UserRole;
use actix_web::{error, Error};

/// Check if user is admin
pub fn is_admin(user: &AuthUserDto) -> bool {
    matches!(user.role, Some(UserRole::ADMIN))
}

/// Check if user is owner of resource or admin
pub fn is_owner_or_admin(user: &AuthUserDto, target_user_id: &str) -> bool {
    is_admin(user) || user.id == target_user_id
}

/// Check permission and return proper Error with message
pub fn check_owner_or_admin(user: &AuthUserDto, target_user_id: &str) -> Result<(), Error> {
    if user.id.is_empty() {
        // Not logged in
        Err(error::ErrorUnauthorized(
            "You must be logged in to perform this action",
        ))
    } else if is_owner_or_admin(user, target_user_id) {
        // Allowed
        Ok(())
    } else {
        // Logged in but not owner/admin
        Err(error::ErrorForbidden(
            "You are not allowed to access or modify this resource",
        ))
    }
}

/// Check only admin access
pub fn check_admin(user: &AuthUserDto) -> Result<(), Error> {
    if is_admin(user) {
        Ok(())
    } else {
        Err(error::ErrorForbidden("Only admins can perform this action"))
    }
}

/// Check if user is admin or school staff
pub fn check_admin_or_staff(user: &AuthUserDto) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) || user.role == Some(UserRole::SCHOOLSTAFF) {
        Ok(())
    } else {
        Err("Insufficient permissions. Requires Admin or School staff role.".to_string())
    }
}

/// Check if user has access to school operations
pub fn check_school_access(user: &AuthUserDto, _school_id: &str) -> Result<(), String> {
    // Admin has full access
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // School staff can access schools they're associated with
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        // Here you might want to check if the staff member belongs to this school
        // For now, we'll allow all school staff to update any school
        // You can implement more granular permissions later
        return Ok(());
    }

    Err("Insufficient permissions to access school".to_string())
}

pub fn check_admin_staff_or_teacher(user: &AuthUserDto) -> Result<(), String> {
    if user.role == Some(UserRole::SCHOOLSTAFF)
        || user.role == Some(UserRole::ADMIN)
        || user.role == Some(UserRole::TEACHER)
    {
        Ok(())
    } else {
        Err("Access denied: Admin, Staff, or Teacher role required".to_string())
    }
}

/// Check if user has access to the class (admin, class teacher, or school staff)
pub fn check_class_access(user: &AuthUserDto, class_id: &str) -> Result<(), String> {
    // Admin has access to everything
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // Check if user is the class teacher for this class
    if user.role == Some(UserRole::TEACHER) {
        // Use unwrap_or_default() to handle Option<Vec<String>>
        if user
            .accessible_classes
            .as_ref()
            .unwrap_or(&Vec::new())
            .contains(&class_id.to_string())
        {
            return Ok(());
        }
    }

    // Check if user is staff member of the school that owns this class
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        // For school staff, we can check if they have access to the school that owns this class
        // This would typically require a database query to get the school_id from the class_id
        // For now, we'll implement a basic check using the schools field
        if let Some(schools) = &user.schools {
            // In a real implementation, you would query the class to get its school_id
            // and check if that school_id is in the user's schools list
            // For now, we'll return true if the user has any schools (placeholder)
            if !schools.is_empty() {
                return Ok(());
            }
        }
    }

    Err("Access denied: No permission to access this class".to_string())
}

/// Check if user is admin or class teacher
pub fn check_admin_or_class_teacher(user: &AuthUserDto, class_id: &str) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    if user.role == Some(UserRole::TEACHER)
        && user
            .accessible_classes
            .as_ref()
            .unwrap_or(&Vec::new())
            .contains(&class_id.to_string())
    {
        return Ok(());
    }

    Err("Access denied: Admin or Class Teacher role required".to_string())
}

// ========== SUBJECT ACCESS CONTROL ==========

/// Check if user has access to subject (Admin, Staff, or Subject Teacher)
pub fn check_subject_access(user: &AuthUserDto, subject_id: &str) -> Result<(), String> {
    // Admin has full access
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // School staff can access subjects
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        return Ok(());
    }

    // For teachers, check if they are the class teacher for this subject
    if user.role == Some(UserRole::TEACHER) && is_subject_teacher(&user.id, subject_id) {
        return Ok(());
    }

    Err("Access denied: insufficient permissions for subject".to_string())
}

/// Check if user is Admin or Subject Teacher
pub fn check_admin_or_subject_teacher(user: &AuthUserDto, subject_id: &str) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    if user.role == Some(UserRole::TEACHER) && is_subject_teacher(&user.id, subject_id) {
        return Ok(());
    }

    Err("Access denied: must be admin or subject teacher".to_string())
}

/// Helper function to check if user is the subject teacher
/// You'll need to implement this based on your database structure
fn is_subject_teacher(user_id: &str, subject_id: &str) -> bool {
    // This is a placeholder implementation
    // You would typically:
    // 1. Parse the subject_id to ObjectId
    // 2. Query the subjects collection to find the subject
    // 3. Check if the user_id matches the class_teacher_id of the subject
    let _ = user_id;
    let _ = subject_id;
    // For now, return false - implement based on your actual data model
    // You might want to use your database connection to query the subject
    // and check if the class_teacher_id matches the user_id

    // Example of how this might look (pseudo-code):
    /*
    use crate::repositories::subject_repo::SubjectRepo;
    use crate::utils::object_id::parse_object_id;
    use crate::models::id_model::IdType;

    let subject_repo = SubjectRepo::new(&db_connection);
    let subject_id_parsed = IdType::from_string(subject_id);

    match subject_repo.find_by_id(&subject_id_parsed).await {
        Ok(Some(subject)) => {
            if let Some(class_teacher_id) = subject.class_teacher_id {
                // Compare the ObjectId with user_id (you might need to convert user_id to ObjectId)
                // This depends on how your user IDs are stored
                class_teacher_id.to_hex() == user_id
            } else {
                false
            }
        }
        _ => false
    }
    */

    // For now, we'll return a simple check based on accessible_classes
    // This assumes that if a teacher has access to a class, they might be teaching subjects in that class
    // Adjust this based on your actual data model
    false
}
