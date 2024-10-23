pub type StaffResult<T> = core::result::Result<T, StaffError>;

#[derive(Debug)]
pub enum StaffError {
    CanNotCreateStaff { error: String },
}

impl std::fmt::Display for StaffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StaffError::CanNotCreateStaff { error } => {
                write!(f, "Can't create Staff error is {}", error)
            }
        }
    }
}
