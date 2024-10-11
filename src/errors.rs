use mongodb::error::Error;

pub type Result<T> = core::result::Result<T , MyError>;

#[derive(Debug)]
pub enum MyError {
    // database error
    InvalidId,
    // school errors
    CantNotCreateSchool,
    // user errors
    UserNotFound,
    UserEmailIsReadyExit {email : String},
    UsernameIsReadyExit {username : String},
    InvalidUserId,
    DatabaseError,
    CreateUserError,
    GetUserErr,
    // user auth
    InvalidCredentials,
    UserNotLoggedIn,
    // image error
    CanNotFindImage,
    // country
    CanNotCreateCountry,
    CanNotFetchCountries,
    // school requested error
    SchoolRequestCanNotCreate,
    SchoolEmailIsLeadExit,
    CanNotFIndSchoolRequest
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // database error
            MyError::InvalidId => write!(f, "Invalid Object ID"),
            MyError::UserNotFound => write!(f, "User not found"),
            MyError::InvalidUserId => write!(f, "Invalid user ID"),
            MyError::CreateUserError => write!(f, " Can't create user"),
            MyError::GetUserErr => write!(f, "Can't get user"),
            MyError::DatabaseError => write!(f, "A database error occurred"),
            MyError::CantNotCreateSchool => write!(f, "Can't create a new school"),
            MyError::UserEmailIsReadyExit {email} => write!(f, "User's email is already registered: {}" , email),
            MyError::UsernameIsReadyExit {username} => write!(f, "username is already exit: {}" , username),
            MyError::UserNotLoggedIn => write!(f, "User is not logged in"),
            MyError::InvalidCredentials => write!(f, "Invalid credentials"),
            MyError::CanNotFindImage => write!(f, "Can not find image"),
            // countries
            MyError::CanNotCreateCountry => write!(f, "Can not create country"),
            MyError::CanNotFetchCountries => write!(f, "Can not Fetches countries"),
            // school request
            MyError::SchoolRequestCanNotCreate => write!(f, "Can't create a new school request"),
            MyError::SchoolEmailIsLeadExit => write!(f, "School email is lead exit "),
            MyError::CanNotFIndSchoolRequest => write!(f, "Can not find a school request"),
        }
    }
}
