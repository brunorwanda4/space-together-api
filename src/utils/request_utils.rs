use actix_web::{HttpMessage, HttpRequest};

use crate::models::school_token_model::SchoolToken;

pub fn get_school_id_from_request(req: &HttpRequest) -> Option<String> {
    req.extensions()
        .get::<SchoolToken>()
        .map(|token| token.id.clone())
}

pub fn get_database_name_from_request(req: &HttpRequest) -> Option<String> {
    req.extensions()
        .get::<SchoolToken>()
        .map(|token| token.database_name.clone())
}