pub type TermResult<T> = core::result::Result<T, TermError>;

#[derive(Debug)]
pub enum TermError {
    CanNotCreateTerm,
    TermInvalidId,
    CanNotGetTerm { error: String },
    TermNotFound,
    NoFieldsToUpdate,
    CanNotUpdateTerm,
}

impl std::fmt::Display for TermError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermError::CanNotCreateTerm => write!(f, "Can not create term"),
            TermError::TermInvalidId => write!(f, "Invalid term ID"),
            TermError::CanNotGetTerm { error } => {
                write!(f, "Can not get term information error is : {:?}", error)
            }
            TermError::TermNotFound => write!(f, "term not found"),
            TermError::NoFieldsToUpdate => {
                write!(f, "No fields to update in term, Please enter a field!")
            }
            TermError::CanNotUpdateTerm => write!(f, "Can not update term"),
        }
    }
}
