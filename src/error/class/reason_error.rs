pub type ReasonResult<T> = core::result::Result<T, ReasonErr>;

#[derive(Debug)]
pub enum ReasonErr {
    InvalidId,
    CanNotCreateReason { error: String },
    CanNotCreateReasonIndex,
    CanNotGetReason { error: String },
    NotFoundReason,
    CanMakeReasonBecauseOfTradingError { error: String },
    NoFieldsToUpdate,
    ReasonNotFound,
    CanNotUpdateReason { error: String },
    CanNotDeleteReason { error: String },
}

impl std::fmt::Display for ReasonErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonErr::InvalidId => write!(f, "Invalid Id for Reason"),
            ReasonErr::CanNotCreateReason { error } => {
                write!(f, "Can't create Reason error is {}", error)
            }
            ReasonErr::CanNotCreateReasonIndex => write!(f, "Can't create Reason index "),
            ReasonErr::CanNotGetReason { error } => {
                write!(f, "Can't get Reason information error is : {:?} ", error)
            }
            ReasonErr::NotFoundReason => write!(f, "Reason not found"),
            ReasonErr::CanMakeReasonBecauseOfTradingError { error } => {
                write!(f, "Can't make Reason because of Trading error : {}", error)
            }
            ReasonErr::NoFieldsToUpdate => {
                write!(f, "No fields to update in Reason, Please enter a field!")
            }
            ReasonErr::ReasonNotFound => write!(f, "Reason not found"),
            ReasonErr::CanNotUpdateReason { error } => {
                write!(f, "Can not update Reason error is: {:?}", error)
            }
            ReasonErr::CanNotDeleteReason { error } => {
                write!(f, "Can not delete Reason error is: {:?}", error)
            }
        }
    }
}
