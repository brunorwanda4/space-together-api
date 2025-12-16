use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

use crate::models::school_token_model::SchoolToken;

pub fn create_school_token(school: SchoolToken) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // valid for 2 hours
        .expect("valid timestamp")
        .timestamp();
    let issued_at = Utc::now().timestamp();

    let claims = SchoolToken {
        id: school.id,
        database_name: school.database_name,
        creator_id: school.creator_id,
        name: school.name,
        username: school.username,
        logo: school.logo,
        school_type: school.school_type,
        affiliation: school.affiliation,
        created_at: school.created_at,
        exp: expiration as usize,
        iat: issued_at as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            env::var("SCHOOL_SECRET")
                .expect("❌ SCHOOL_SECRET not set in .env")
                .as_bytes(),
        ),
    )
    .expect("Failed to encode school token")
}

pub fn verify_school_token(token: &str) -> Option<SchoolToken> {
    match decode::<SchoolToken>(
        token,
        &DecodingKey::from_secret(
            env::var("SCHOOL_SECRET")
                .expect("❌ SCHOOL_SECRET not set in .env")
                .as_bytes(),
        ),
        &Validation::default(),
    ) {
        Ok(data) => Some(data.claims),
        Err(_) => None,
    }
}
