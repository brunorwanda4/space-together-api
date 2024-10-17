pub type TeamResult<T> = core::result::Result<T, TeamError>;

#[derive(Debug)]
pub enum TeamError {
    CanNotCreateTeam,
}

impl std::fmt::Display for TeamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamError::CanNotCreateTeam => write!(f, "Can not create team"),
        }
    }
}
