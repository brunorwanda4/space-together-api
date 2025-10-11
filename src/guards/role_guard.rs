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
