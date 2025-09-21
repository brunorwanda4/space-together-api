use crate::domain::{auth_user::AuthUserDto, user::User};

pub fn to_auth_dto(user: &User) -> AuthUserDto {
    AuthUserDto {
        id: user
            .id
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_default(),
        name: user.name.clone(),
        email: user.email.clone(),
        username: user.username.clone().unwrap_or_default(),
        image: user.image.clone(),
        phone: user.phone.clone(),
        role: user.role.clone(),
        gender: user.gender.clone(),
        disable: user.disable,
        current_school_id: Some(
            user.current_school_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        ),
        iat: None,
        exp: None,
    }
}
