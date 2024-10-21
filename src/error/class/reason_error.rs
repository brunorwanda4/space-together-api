pub type ReasonResult<T> = core::result::Result<T, ReasonErr>;

#[derive(Debug)]
pub enum ReasonErr {
    InvalidId,
    CanNotCreateReason { error: String },
    CanNotGetReason { error: String },
    NotFoundReason,
    NoFieldsToUpdate,
}

impl std::fmt::Display for ReasonErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonErr::InvalidId => write!(f, "Invalid Id for Reason"),
            ReasonErr::CanNotCreateReason { error } => {
                write!(f, "Can't create Reason error is {}", error)
            }
            ReasonErr::CanNotGetReason { error } => {
                write!(f, "Can't get Reason information error is : {:?} ", error)
            }
            ReasonErr::NoFieldsToUpdate => {
                write!(f, "No Fields to update Reason , please field all data")
            }
            ReasonErr::NotFoundReason => write!(f, "Reason not found"),
        }
    }
}
