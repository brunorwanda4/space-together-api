pub type StaffResult<T> = core::result::Result<T, StaffError>;

#[derive(Debug)]
pub enum StaffError {
    CanNotCreateStaff { error: String },
    StaffIsReadyExit,
    InvalidId,
    CanNotFindStaffById { error: String },
    StaffNotFound,
    UserNotFound,
    UserIsReadyExit,
    CanNotFindStaffByUserId { error: String },
    UserIdNotFound,
    UserIdIsReadyExit,
}

impl std::fmt::Display for StaffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StaffError::CanNotCreateStaff { error } => {
                write!(f, "Can't create Staff error is: {}", error)
            }
            StaffError::StaffIsReadyExit => write!(f, "Staff is ready to exit "),
            StaffError::InvalidId => write!(f, "Invalid Id for Staff School"),
            StaffError::CanNotFindStaffById { error } => {
                write!(f, "Staff is not not found error is: {}", error)
            }
            StaffError::StaffNotFound => write!(f, "Staff not found by id"),
            StaffError::UserNotFound => write!(f, "You not found, please try again later"),
            StaffError::UserIsReadyExit => write!(f, "user is ready exit "),
            StaffError::CanNotFindStaffByUserId { error } => {
                write!(f, " Can't find staff by user id because : {}", error)
            }
            StaffError::UserIdNotFound => write!(f, " user id not found "),
            StaffError::UserIdIsReadyExit => write!(f, " user id is ready exit , try other"),
        }
    }
}
