pub type SchoolRequestResult<T> = core::result::Result<T, SchoolRequestErr>;

#[derive(Debug)]
pub enum SchoolRequestErr {
    CanNotCreateSchoolRequest,
    CanNotGetSchoolRequest,
}

impl std::fmt::Display for SchoolRequestErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchoolRequestErr::CanNotCreateSchoolRequest => write!(f, "Can not create school"),
            SchoolRequestErr::CanNotGetSchoolRequest => {
                write!(f, "Can not get school request information ")
            }
        }
    }
}
