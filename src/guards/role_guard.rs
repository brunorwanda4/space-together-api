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
