pub type AvatarResult<T> = core::result::Result<T, AvatarError>;

#[derive(Debug)]
pub enum AvatarError {
    AvatarUserIdIsReadyExit,
}

impl std::fmt::Display for AvatarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvatarError::AvatarUserIdIsReadyExit => write!(f, "Avatar user id ready exit"),
        }
    }
}
