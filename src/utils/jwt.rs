use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::domain::auth_user::AuthUserDto;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user: AuthUserDto, // instead of just `sub`
    pub exp: usize,
    pub iat: usize,
}

pub fn create_jwt(user: &AuthUserDto) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let issued_at = Utc::now().timestamp();

    // clone user but override iat/exp
    let mut user_with_times = user.clone();
    user_with_times.iat = Some(issued_at);
    user_with_times.exp = Some(expiration);

    let claims = Claims {
        user: user_with_times,
        exp: expiration as usize,
        iat: issued_at as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            env::var("APP_SECRET")
                .expect("❌ APP_SECRET not set in .env")
                .as_bytes(),
        ),
    )
    .expect("Failed to encode JWT")
}

pub fn verify_jwt(token: &str) -> Option<Claims> {
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(
            env::var("APP_SECRET")
                .expect("❌ APP_SECRET not set in .env")
                .as_bytes(),
        ),
        &Validation::default(),
    ) {
        Ok(data) => Some(data.claims),
        Err(_) => None,
    }
}
