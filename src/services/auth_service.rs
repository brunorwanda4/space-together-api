use crate::{
    config::state::AppState, domain::{
        auth::{LoginResponse, LoginUser, RegisterUser},
        common_details::UserRole,
        user::{UpdateUserDto, User},
    }, errors::AppError, mappers::user_mapper::to_auth_dto, models::id_model::IdType, repositories::user_repo::UserRepo, services::{school_service::SchoolService, user_service::UserService}, utils::{
        email::is_valid_email,
        hash::verify_password,
        jwt::{create_jwt, verify_jwt},
        names::{generate_username, is_valid_name, is_valid_username},
        user_utils::sanitize_user,
    }
};
use chrono::Utc;
use mongodb::bson::doc;

pub struct AuthService<'a> {
    repo: &'a UserRepo,
}

impl<'a> AuthService<'a> {
    pub fn new(repo: &'a UserRepo) -> Self {
        Self { repo }
    }

    pub async fn get_auth_user(&self, user_email: &str, password:Option<&String>,  state: &AppState) -> Result<LoginResponse, AppError> {
        let db = state.db.main_db();
        
        // allow user to login with username or email
        let user =match is_valid_email(user_email) {
            Ok(email) => match self
            .repo
            .find_by_email(&email).await? {
                None => return Err(AppError { message: "User not found by email".to_string() }),
                Some(u) => u
            },
            Err(_error) =>match is_valid_username(&user_email) {
                Ok(username) =>  match self.repo.find_by_username(&username).await? {
               None => return Err(AppError { message: "User not found by username".to_string() }),
                Some(u) => u 
            }, 
            Err(username_error) => return Err(AppError {message: username_error})
            }
        };

      

            if let Some(user_password) = password {
            let hash = user
            .password_hash
            .as_ref()
            .ok_or(AppError{message:"This account does not have a password set".to_string()})?;

                  if !verify_password(hash, user_password) {
            return Err(AppError{message: "Incorrect password, please try again".into()});
        }
            }

        let school_service = SchoolService::new(&db);
        let mut current_school_user_id = None;
        let mut school_access_token = None;

        // Handle school-specific data if user has a current school
        if let Some(ref school_id) = user.current_school_id {
            let school_db_name = format!("school_{}", school_id.to_string());
            let school_db = state.db.get_db(&school_db_name);

            // Get user ObjectId
            let user_id = user
                .id
                .as_ref()
                .ok_or_else(|| AppError {
                    message: "User does not have an ID".to_string(),
                })?
                .clone();

            let member_type = user.role.clone();

            // Search for school member
            let school_member = school_service
                .search_single_member(
                    &school_db,
                    None,
                    Some(doc! {"user_id": user_id}),
                    member_type,
                )
                .await?;

            current_school_user_id = school_member.get_id();

            // Create auth DTO with school user ID
            let auth_user_dto = to_auth_dto(&user, current_school_user_id.clone());

            // Generate school access token
            school_access_token = Some(
                school_service
                    .create_school_token(&IdType::from_object_id(school_id.clone()), &auth_user_dto, state)
                    .await?,
            );
        }

        // Create main auth DTO and access token
        let auth_user_dto = to_auth_dto(&user, current_school_user_id.clone());
        let access_token = create_jwt(&auth_user_dto);

        Ok(LoginResponse {
            id: user.id.map(|i| i.to_string()),
            email: user.email,
            name: user.name,
            access_token,
            image: user.image,
            role: user.role,
            username: user.username,
            bio: user.bio,
            current_school_user_id,
            schools: user
                .schools
                .map(|ids| ids.into_iter().map(|id| id.to_string()).collect()),
            current_school_id: user.current_school_id.map(|id| id.to_string()),
            school_access_token,
        })
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

        let user = User {
            id: None,
            name: valid_name,
            email: data.email,
            username,
            password_hash: Some(data.password),
            role: Some(UserRole::STUDENT),
            image_id: None,
            image: None,
            background_images: None,
            bio: None,
            disable: None,
            phone: None,
            address: None,
            social_media: None,
            preferred_communication_method: None,
            gender: None,
            age: None,
            languages_spoken: None,
            hobbies_interests: None,
            dream_career: None,
            special_skills: None,
            health_or_learning_notes: None,
            current_school_id: None,
            schools: None,
            accessible_classes: None,
            favorite_subjects_category: None,
            preferred_study_styles: None,
            guardian_info: None,
            special_support_needed: None,
            learning_challenges: None,
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
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        // 💾 Save user
        let res = user_service
            .create_user(user)
            .await
            .map_err(|e| e.to_string())?;

        // 🪪 Generate token
        let dto = to_auth_dto(&sanitize_user(res.clone()), None);
        let token = create_jwt(&dto);

        Ok((token, sanitize_user(res)))
    }

    /// ✅ Log in existing user
    pub async fn login(&self, data: LoginUser, state: &AppState) -> Result<LoginResponse, AppError> {
self.get_auth_user(&data.email, Some(&data.password), state).await
      
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
        let dto = to_auth_dto(&sanitize_user(updated_user.clone()), None);
        let new_token = create_jwt(&dto);

        Ok((new_token, sanitize_user(updated_user)))
    }

    /// 🔄 Refresh JWT token if still valid
    pub async fn refresh_token(&self, token: &str ,state: &AppState) -> Result<String, AppError> {
        // remove "Bearer " if present
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_jwt(&token_clean).ok_or_else(|| AppError{message: "Invalid token".to_string()})?;

        let auth = self.get_auth_user(&claims.user.email,None, state).await?;

        Ok(auth.access_token)
    }
}
