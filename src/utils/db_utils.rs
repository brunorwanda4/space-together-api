use crate::config::state::AppState;
use crate::models::school_token_model::SchoolToken;
use actix_web::{web, HttpMessage, HttpRequest};
use mongodb::Database;

/// Get the appropriate database based on school context
/// - If SchoolToken exists in request extensions, returns school-specific database
/// - Otherwise returns main database (for admins, cross-school conversations, etc.)
pub fn get_database(req: &HttpRequest, state: &web::Data<AppState>) -> Database {
    if let Some(claims) = req.extensions().get::<SchoolToken>() {
        return state.db.get_db(&claims.database_name);
    }
    state.db.main_db()
}
