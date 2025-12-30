use crate::config::state::AppState;
use crate::models::school_token_model::SchoolToken;
use actix_web::{web, HttpMessage, HttpRequest};
use mongodb::Database;

pub fn get_database(req: &HttpRequest, state: &web::Data<AppState>) -> Database {
    if let Some(claims) = req.extensions().get::<SchoolToken>() {
        return state.db.get_db(&claims.database_name);
    }
    state.db.main_db()
}
