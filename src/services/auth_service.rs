use crate::{
    domain::{
        auth::{LoginUser, RegisterUser},
        user::{UpdateUserDto, User},
        user_role::UserRole,
    },
    mappers::user_mapper::to_auth_dto,
    models::id_model::IdType,
    repositories::user_repo::UserRepo,
    utils::{
        email::is_valid_email,
        hash::{hash_password, verify_password},
        jwt::{create_jwt, verify_jwt},
        names::{generate_username, is_valid_name},
    },
};
use chrono::Utc;

pub struct AuthService<'a> {
    repo: &'a UserRepo,
}

impl<'a> AuthService<'a> {
    pub fn new(repo: &'a UserRepo) -> Self {
        Self { repo }
    }

    pub async fn register(&self, data: RegisterUser) -> Result<(String, User), String> {
        is_valid_email(&data.email)?;

        if let Ok(Some(_)) = self.repo.find_by_email(&data.email).await {
            return Err("Email already exists".to_string());
        }

        let valid_name = is_valid_name(&data.name)?;
        let username = Some(generate_username(&valid_name));
        let now = Some(Utc::now());

        let user = User {
            id: None,
            name: valid_name,
            email: data.email,
            username,
            password_hash: Some(hash_password(&data.password)),
            role: Some(UserRole::STUDENT),
            image: None,
            image_id: None,
            phone: None,
            gender: None,
            age: None,
            address: None,
            current_school_id: None,
            disable: None,
            bio: None,
            created_at: now,
            updated_at: now,
        };

        let res = self
            .repo
            .insert_user(&user)
            .await
            .map_err(|e| e.to_string())?;

        let dto = to_auth_dto(&res);
        let token = create_jwt(&dto);

        Ok((token, res))
    }

    pub async fn login(&self, data: LoginUser) -> Result<(String, User), String> {
        let user = self
            .repo
            .find_by_email(&data.email)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(user) = user {
            if let Some(ref hash) = user.password_hash {
                if verify_password(hash, &data.password) {
                    let dto = to_auth_dto(&user);
                    let token = create_jwt(&dto);
                    return Ok((token, user));
                }
            }
            Err("Invalid credentials".to_string())
        } else {
            Err("User not found".to_string())
        }
    }

    pub async fn get_user_from_token(&self, token: &str) -> Result<User, String> {
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_jwt(&token_clean).ok_or_else(|| "Invalid token".to_string())?;
        let user_id = &claims.user.id;

        match self.repo.find_by_id(&IdType::from_string(user_id)).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err("User not found".to_string()),
            Err(e) => Err(e.message),
        }
    }

    pub async fn onboard_user(
        &self,
        user_id: &str,
        updated_data: UpdateUserDto,
        user_service: &crate::services::user_service::UserService<'a>,
    ) -> Result<(String, crate::domain::user::User), String> {
        let id = crate::models::id_model::IdType::from_string(user_id);

        // update user
        let updated_user = user_service.update_user(&id, updated_data).await?;

        // issue fresh token
        let dto = crate::mappers::user_mapper::to_auth_dto(&updated_user);
        let new_token = crate::utils::jwt::create_jwt(&dto);

        Ok((new_token, updated_user))
    }

    pub async fn refresh_token(&self, token: &str) -> Result<String, String> {
        // remove "Bearer " if present
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_jwt(&token_clean).ok_or_else(|| "Invalid token".to_string())?;

        // get user from DB to ensure still valid
        let user_id = &claims.user.id;
        let user = self
            .repo
            .find_by_id(&crate::models::id_model::IdType::from_string(user_id))
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User not found".to_string())?;

        // create a fresh token
        let dto = crate::mappers::user_mapper::to_auth_dto(&user);
        let new_token = crate::utils::jwt::create_jwt(&dto);

        Ok(new_token)
    }
}
