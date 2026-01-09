use mongodb::bson::oid::ObjectId;

use crate::{errors::AppError, };



pub fn parse_object_id_value(value: &str) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(value).map_err(|e| AppError {
        message: format!("Invalid ObjectId: {}", e),
    })
}
