use crate::domain::user::User;

/// Remove password hash from a user before returning
pub fn sanitize_user(mut user: User) -> User {
    user.password_hash = None;
    user
}

/// Remove password hash from a vector of users
pub fn sanitize_users(users: Vec<User>) -> Vec<User> {
    users.into_iter().map(sanitize_user).collect()
}
