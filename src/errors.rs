use actix_web::{HttpResponse, ResponseError};
use mongodb::error::Error as MongoError;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Implement ResponseError for actix_web
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": self.message
        }))
    }
}

// ✅ Implement From<MongoError> separately
impl From<MongoError> for AppError {
    fn from(err: MongoError) -> Self {
        AppError {
            message: format!("MongoDB Error: {}", err),
        }
    }
}
