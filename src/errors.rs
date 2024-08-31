pub type Result<T> = core::result::Result<T , MyError>;

#[derive(Debug)]
pub enum MyError {
    // school errors
    CantNotCreateSchool,

    // user errors
    UserNotFound,
    UserEmailIsReadyExit,
}
