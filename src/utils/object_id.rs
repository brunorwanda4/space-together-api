use mongodb::bson::oid::ObjectId;

use crate::{errors::AppError, models::id_model::IdType};

pub fn parse_object_id(id: &IdType) -> Result<ObjectId, AppError> {
    ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
        message: format!("Invalid ObjectId: {}", e),
    })
}
