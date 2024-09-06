use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Error, Json};
use bcrypt::verify;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{errors::MyError, handlers::AUTH_TOKEN, models::user_model::LoginUserModel};
use crate::AppState;

pub async fn user_login_handler(
    cookies : Cookies,
    Json(user) : Json<LoginUserModel>,
    State(app_state) : State<Arc<AppState>>,
)
 -> Result<Json<Value>, (StatusCode)>{
    let get_user = app_state.db.get_user_by_email(user.email).await.unwrap();

    if get_user.password.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let hash_password = get_user.password.unwrap();
    let password = user.password;

    let compier_password = verify( password, &hash_password).unwrap();
    if !compier_password {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    cookies.add(Cookie::new(AUTH_TOKEN , "user-1.exp.sign"));

    let body = Json(json!({
        "result" : {
            "success" : true,
        }
    }));

    Ok(body)
}