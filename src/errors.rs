use mongodb::error::Error;

pub type Result<T> = core::result::Result<T , MyError>;

#[derive(Debug)]
pub enum MyError {
    // school errors
    CantNotCreateSchool,
    // user errors
    UserNotFound,
    UserEmailIsReadyExit {email : String},
    InvalidUserId,
    DatabaseError,
    CreateUserError,
    GetUserErr,
    // user auth
    InvalidCredentials,
    UserNotLoggedIn,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::UserNotFound => write!(f, "User not found"),
            MyError::InvalidUserId => write!(f, "Invalid user ID"),
            MyError::CreateUserError => write!(f, " Can't create user"),
            MyError::GetUserErr => write!(f, "Can't get user"),
            MyError::DatabaseError => write!(f, "A database error occurred"),
            MyError::CantNotCreateSchool => write!(f, "Can't create a new school"),
            MyError::UserEmailIsReadyExit {email} => write!(f, "User's email is already registered: {}" , email),
            MyError::UserNotLoggedIn => write!(f, "User is not logged in"),
            MyError::InvalidCredentials => write!(f, "Invalid credentials"),
        }
    }
}
