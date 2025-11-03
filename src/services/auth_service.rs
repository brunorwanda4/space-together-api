use crate::{
    domain::{
        auth::{LoginUser, RegisterUser},
        common_details::UserRole,
        user::{UpdateUserDto, User},
    },
    mappers::user_mapper::to_auth_dto,
    models::id_model::IdType,
    repositories::user_repo::UserRepo,
    services::user_service::UserService,
    utils::{
        email::is_valid_email,
        hash::{hash_password, verify_password},
        jwt::{create_jwt, verify_jwt},
        names::{generate_username, is_valid_name},
        user_utils::sanitize_user,
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

    /// ✅ Register a new user
    pub async fn register(
        &self,
        user_service: &UserService<'a>,
        data: RegisterUser,
    ) -> Result<(String, User), String> {
        is_valid_email(&data.email)?;

        // 🔒 Ensure email not already taken
        if let Ok(Some(_)) = self.repo.find_by_email(&data.email).await {
            return Err("Email already exists".to_string());
        }

        // 🧩 Validate name and generate username
        let valid_name = is_valid_name(&data.name)?;
        let username = Some(generate_username(&valid_name));
        let now = Some(Utc::now());

        // 🧱 Create new user record (fill all fields)
        let user = User {
            // 🔹 Identification & Authentication
            id: None,
            name: valid_name,
            email: data.email,
            username,
            password_hash: Some(hash_password(&data.password)),
            role: Some(UserRole::STUDENT),

            // 🔹 Profile & Media
            image_id: None,
            image: None,
            background_images: None,
            bio: None,
            disable: Some(false),

            // 🔹 Contact & Social
            phone: None,
            address: None,
            social_media: None,
            preferred_communication_method: None,

            // 🔹 Personal Info
            gender: None,
            age: None,
            languages_spoken: None,
            hobbies_interests: None,
            dream_career: None,
            special_skills: None,
            health_or_learning_notes: None,

            // 🔹 Academic & School
            current_school_id: None,
            schools: None,
            accessible_classes: None,
            favorite_subjects_category: None,
            preferred_study_styles: None,

            // 🔹 Guardian, Support, Learning
            guardian_info: None,
            special_support_needed: None,
            learning_challenges: None,

            // 🔹 Teaching & Employment
            teaching_level: None,
            employment_type: None,
            teaching_start_date: None,
            years_of_experience: None,
            education_level: None,
            certifications_trainings: None,
            preferred_age_group: None,
            professional_goals: None,
            availability_schedule: None,
            department: None,
            job_title: None,
            teaching_style: None,

            // 🔹 Timestamps
            created_at: now,
            updated_at: now,
        };
        // 💾 Save user
        let res = user_service
            .create_user(user)
            .await
            .map_err(|e| e.to_string())?;

        // 🪪 Generate token
        let dto = to_auth_dto(&sanitize_user(res.clone()));
        let token = create_jwt(&dto);

        Ok((token, sanitize_user(res)))
    }

    /// ✅ Log in existing user
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
                    return Ok((token, sanitize_user(user)));
                }
            }
            Err("Invalid credentials".to_string())
        } else {
            Err("User not found".to_string())
        }
    }

    /// ✅ Get user info from JWT
    pub async fn get_user_from_token(&self, token: &str) -> Result<User, String> {
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_jwt(&token_clean).ok_or_else(|| "Invalid token".to_string())?;
        let user_id = &claims.user.id;

        match self.repo.find_by_id(&IdType::from_string(user_id)).await {
            Ok(Some(user)) => Ok(sanitize_user(user)),
            Ok(None) => Err("User not found".to_string()),
            Err(e) => Err(e.message),
        }
    }

    /// ✅ Onboard user (update profile during first setup)
    pub async fn onboard_user(
        &self,
        user_id: &str,
        updated_data: UpdateUserDto,
        user_service: &UserService<'a>,
    ) -> Result<(String, User), String> {
        let id = IdType::from_string(user_id);

        // 🔄 Update user partially (using new UpdateUserDto)
        let updated_user = user_service.update_user(&id, updated_data).await?;

        // 🪪 Issue fresh token
        let dto = to_auth_dto(&sanitize_user(updated_user.clone()));
        let new_token = create_jwt(&dto);

        Ok((new_token, sanitize_user(updated_user)))
    }

    /// 🔄 Refresh JWT token if still valid
    pub async fn refresh_token(&self, token: &str) -> Result<String, String> {
        // remove "Bearer " if present
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_jwt(&token_clean).ok_or_else(|| "Invalid token".to_string())?;

        // get user from DB to ensure still valid
        let user_id = &claims.user.id;
        let user = self
            .repo
            .find_by_id(&IdType::from_string(user_id))
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User not found".to_string())?;

        // create a fresh token
        let dto = to_auth_dto(&sanitize_user(user));
        let new_token = create_jwt(&dto);

        Ok(new_token)
    }
}
