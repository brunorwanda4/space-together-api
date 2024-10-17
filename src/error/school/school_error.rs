pub type SchoolResult<T> = core::result::Result<T, SchoolErr>;

#[derive(Debug)]
pub enum SchoolErr {
    CanNotCreateSchool,
}

impl std::fmt::Display for SchoolErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchoolErr::CanNotCreateSchool => write!(f, "Can not create school"),
        }
    }
}
