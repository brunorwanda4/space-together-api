use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Error, Json};
use bcrypt::verify;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{errors::MyError, handlers::AUTH_TOKEN, models::user_model::LoginUserModel};
use crate::AppState;

pub async fn user_login_controller(
    cookies : Cookies,
    Json(user) : Json<LoginUserModel>,
    State(app_state) : State<Arc<AppState>>,
)
 -> Result<Json<Value>, (StatusCode)>{
    let get_user = app_state
        .db
        .get_user_by_email(user.email.clone())
        .await
        .map_err(|_| MyError::UserEmailIsReadyExit { email : user.email.clone() });

    let user = match get_user {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if user.password.is_some() {
        // let hash_password = user
    }

    // let hash_password = user.password.inspect("user password is invalid");
    
    todo!()
}