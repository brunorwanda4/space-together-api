pub type TermResult<T> = core::result::Result<T, TermError>;

#[derive(Debug)]
pub enum TermError {
    CanNotCreateterm,
    TermInvalidId,
    CanNotGetterm { error: String },
    TermNotFound,
    NoFieldsToUpdate,
    CanNotUpdateterm,
}

impl std::fmt::Display for TermError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermError::CanNotCreateterm => write!(f, "Can not create term"),
            TermError::TermInvalidId => write!(f, "Invalid term ID"),
            TermError::CanNotGetterm { error } => {
                write!(f, "Can not get term information error is : {:?}", error)
            }
            TermError::TermNotFound => write!(f, "term not found"),
            TermError::NoFieldsToUpdate => {
                write!(f, "No fields to update in term, Please enter a field!")
            }
            TermError::CanNotUpdateterm => write!(f, "Can not update term"),
        }
    }
}
