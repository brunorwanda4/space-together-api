use mongodb::error::Error;
pub type Result<T> = core::result::Result<T , Error>;

pub enum MyError {
    // school errors
    CantNotCreateSchool,
    MongoErrorQuery(Error),
    UserNotFound,
}
