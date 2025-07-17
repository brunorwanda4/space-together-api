pub type UserResult<T> = core::result::Result<T, UserError>;

#[derive(Debug)]
pub enum UserError {
    CanNotCreateUser { error: String },
    UserEmailOrUsernameIsReadyExit,
    UserEmailIsReadyExit,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::CanNotCreateUser { error } => {
                write!(f, "Can't create user error is : {}", error)
            }
            UserError::UserEmailOrUsernameIsReadyExit => {
                write!(f, " user email or username is ready Exit")
            }
            UserError::UserEmailIsReadyExit => write!(f, " Your email is ready Exit, try other"),
        }
    }
}
