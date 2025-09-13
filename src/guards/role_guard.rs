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
