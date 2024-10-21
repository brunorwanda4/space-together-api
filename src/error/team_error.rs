pub type TeamResult<T> = core::result::Result<T, TeamError>;

#[derive(Debug)]
pub enum TeamError {
    CanNotCreateTeam,
    TeamInvalidId,
    CanNotGetTeam,
    TeamNotFound,
}

impl std::fmt::Display for TeamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamError::CanNotCreateTeam => write!(f, "Can not create team"),
            TeamError::TeamInvalidId => write!(f, "Invalid team ID"),
            TeamError::CanNotGetTeam => write!(f, "Can not get team information"),
            TeamError::TeamNotFound => write!(f, "Team not found"),
        }
    }
}
