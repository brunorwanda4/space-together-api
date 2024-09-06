pub type Result<T> = core::result::Result<T , MyError>;

#[derive(Debug)]
pub enum MyError {
    // school errors
    CantNotCreateSchool,
    // user errors
    UserNotFound,
    UserEmailIsReadyExit,
    InvalidUserId,
    DatabaseError,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MyError::UserNotFound => write!(f, "User not found"),
            MyError::InvalidUserId => write!(f, "Invalid user ID"),
            MyError::DatabaseError => write!(f, "A database error occurred"),
            MyError::CantNotCreateSchool => write!(f, "Can't create a new school"),
            MyError::UserEmailIsReadyExit => write!(f, "User's email is already registered"),
        }
    }
}
