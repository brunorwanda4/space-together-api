use actix_web::web;
use chrono::{Datelike, Utc};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::{
        join_school_request::{
            BulkCreateJoinSchoolRequest, BulkRespondRequest, CreateJoinSchoolRequest,
            JoinRequestQuery, JoinRole, JoinSchoolRequest, JoinStatus, RespondToJoinRequest,
            UpdateRequestExpiration,
        },
        school::School,
        school_staff::{SchoolStaff, SchoolStaffType},
        student::{Student, StudentStatus},
        teacher::{Teacher, TeacherType},
        user::User,
    },
    errors::AppError,
    models::id_model::IdType,
    repositories::join_school_request_repo::JoinSchoolRequestRepo,
    repositories::{
        school_staff_repo::SchoolStaffRepo, student_repo::StudentRepo, teacher_repo::TeacherRepo,
    },
    services::{
        school_service::SchoolService, school_staff_service::SchoolStaffService,
        student_service::StudentService, teacher_service::TeacherService,
        user_service::UserService,
    },
    utils::email::is_valid_email,
};

pub struct JoinSchoolRequestController<'a> {
    pub join_request_repo: JoinSchoolRequestRepo,
    pub user_service: &'a UserService<'a>,
    pub school_service: &'a SchoolService<'a>,
    pub teacher_service: &'a TeacherService<'a>,
    pub student_service: &'a StudentService<'a>,
    pub staff_service: &'a SchoolStaffService<'a>,
}

impl<'a> JoinSchoolRequestController<'a> {
    pub fn new(
        join_request_repo: JoinSchoolRequestRepo,
        user_service: &'a UserService<'a>,
        school_service: &'a SchoolService<'a>,
        teacher_service: &'a TeacherService<'a>,
        student_service: &'a StudentService<'a>,
        staff_service: &'a SchoolStaffService<'a>,
    ) -> Self {
        Self {
            join_request_repo,
            user_service,
            school_service,
            teacher_service,
            student_service,
            staff_service,
        }
    }

    // ----------------------------------------------------------------------
    // Create a new join school request
    // ----------------------------------------------------------------------
    pub async fn create_join_request(
        &self,
        create_request: CreateJoinSchoolRequest,
        sent_by: ObjectId,
    ) -> Result<JoinSchoolRequest, AppError> {
        // Validate email
        if let Err(e) = is_valid_email(&create_request.email) {
            return Err(AppError {
                message: format!("Invalid email: {}", e),
            });
        }

        // Parse school_id and check existence
        let school_id: IdType = IdType::String(create_request.school_id.clone());
        let school_obj_id = self
            .school_service
            .parse_school_id(&school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        let _school = self
            .school_service
            .get_school_by_id(&school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        // Validate staff role limits
        if let JoinRole::Staff = create_request.role {
            let staff_type = self.parse_staff_type(&create_request.r#type);

            match staff_type {
                SchoolStaffType::Director => {
                    let count = self
                        .staff_service
                        .count_school_staff_by_type(SchoolStaffType::Director)
                        .await
                        .unwrap_or(0);

                    if count >= 1 {
                        return Err(AppError {
                            message: "This school already has a Director".into(),
                        });
                    }
                }
                SchoolStaffType::HeadOfStudies => {
                    let count = self
                        .staff_service
                        .count_school_staff_by_type(SchoolStaffType::HeadOfStudies)
                        .await
                        .unwrap_or(0);
                    if count >= 5 {
                        return Err(AppError {
                            message: "This school already has 5 HeadOfStudies".into(),
                        });
                    }
                }
            }
        }

        // Duplicate check
        if let Some(_) = self
            .join_request_repo
            .find_pending_by_email_and_school(&create_request.email, &school_id)
            .await?
        {
            return Err(AppError {
                message: "A pending join request already exists for this email and school".into(),
            });
        }

        // User already in school check
        if let Ok(user) = self
            .user_service
            .get_user_by_email(&create_request.email)
            .await
        {
            if let Some(schools) = &user.schools {
                if schools.contains(&school_obj_id) {
                    return Err(AppError {
                        message: "User is already a member of this school".into(),
                    });
                }
            }
        }

        // Create the join request
        let now = Utc::now();
        let join_request = JoinSchoolRequest {
            id: None,
            school_id: school_obj_id,
            invited_user_id: None,
            role: create_request.role.clone(),
            email: create_request.email.clone(),
            r#type: create_request.r#type.clone(),
            message: create_request.message,
            status: JoinStatus::Pending,
            sent_at: now,
            responded_at: None,
            expires_at: Some(now + chrono::Duration::days(7)),
            sent_by,
            created_at: now,
            updated_at: now,
        };

        let created_request = self.join_request_repo.create(join_request).await?;
        Ok(created_request)
    }

    // ----------------------------------------------------------------------
    // Bulk create join requests
    // ----------------------------------------------------------------------
    pub async fn bulk_create_join_requests(
        &self,
        bulk_request: BulkCreateJoinSchoolRequest,
        sent_by: ObjectId,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let mut requests_to_create = Vec::new();

        for create_request in bulk_request.requests {
            if is_valid_email(&create_request.email).is_err() {
                continue;
            }

            let school_id = IdType::String(create_request.school_id.clone());
            let school_obj_id = match self.school_service.parse_school_id(&school_id).await {
                Ok(id) => id,
                Err(_) => continue,
            };

            if let Ok(Some(_)) = self
                .join_request_repo
                .find_pending_by_email_and_school(&create_request.email, &school_id)
                .await
            {
                continue;
            }

            let join_request = JoinSchoolRequest {
                id: None,
                school_id: school_obj_id,
                invited_user_id: None,
                role: create_request.role.clone(),
                email: create_request.email.clone(),
                r#type: create_request.r#type.clone(),
                message: create_request.message,
                status: JoinStatus::Pending,
                sent_at: Utc::now(),
                responded_at: None,
                expires_at: Some(Utc::now() + chrono::Duration::days(7)),
                sent_by,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            requests_to_create.push(join_request);
        }

        if requests_to_create.is_empty() {
            return Err(AppError {
                message: "No valid requests to create, it should be requests are exits".into(),
            });
        }

        let created_requests = self
            .join_request_repo
            .bulk_create(requests_to_create)
            .await?;
        Ok(created_requests)
    }

    // ----------------------------------------------------------------------
    // Accept join request (creates entity in school DB)
    // ----------------------------------------------------------------------
    pub async fn accept_join_request(
        &self,
        respond_request: RespondToJoinRequest,
        accepted_by: Option<ObjectId>,
        state: web::Data<AppState>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let request_id = IdType::String(respond_request.request_id.clone());
        let request = self
            .join_request_repo
            .find_by_id(&request_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Join request not found".into(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Join request is not pending".into(),
            });
        }

        // Get school & its DB
        let school_id = IdType::ObjectId(request.school_id);
        let school = self
            .school_service
            .get_school_by_id(&school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        let school_db_name = school.database_name.as_ref().ok_or_else(|| AppError {
            message: "School database not configured".into(),
        })?;
        let school_db = state.db.get_db(school_db_name);

        // Get user
        let user = self
            .user_service
            .get_user_by_email(&request.email)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find user: {}", e),
            })?;

        // Create entity in school DB
        self.create_role_entity_school_db(&request, &user, &school, &school_db)
            .await?;

        // Link user to school (in main DB)
        self.user_service
            .add_school_to_user(&IdType::ObjectId(user.id.unwrap()), &school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        // Mark request as accepted
        let updated_request = self
            .join_request_repo
            .accept_request(&request_id, user.id.unwrap(), accepted_by)
            .await?;

        Ok(updated_request)
    }

    // ----------------------------------------------------------------------
    // Create entity inside school database (student/teacher/staff)
    // ----------------------------------------------------------------------
    async fn create_role_entity_school_db(
        &self,
        request: &JoinSchoolRequest,
        user: &User,
        school: &School,
        school_db: &mongodb::Database,
    ) -> Result<(), AppError> {
        let user_id = user.id.unwrap();
        let school_id = request.school_id;

        match request.role {
            JoinRole::Student => {
                let student_repo = StudentRepo::new(school_db);
                let student_service = StudentService::new(&student_repo);

                let student = Student {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    class_id: None,
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    phone: user.phone.clone(),
                    gender: user.gender.clone(),
                    date_of_birth: user.age.clone(),
                    registration_number: self.generate_registration_number(school).await,
                    admission_year: Some(Utc::now().year()),
                    status: StudentStatus::Active,
                    is_active: true,
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                student_service
                    .create_student(student)
                    .await
                    .map_err(|e| AppError { message: e })?;
            }
            JoinRole::Teacher => {
                let teacher_repo = TeacherRepo::new(school_db);
                let teacher_service = TeacherService::new(&teacher_repo);

                let teacher = Teacher {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    phone: user.phone.clone(),
                    gender: user.gender.clone(),
                    r#type: self.parse_teacher_type(&request.r#type),
                    class_ids: None,
                    subject_ids: None,
                    is_active: true,
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                teacher_service
                    .create_teacher(teacher)
                    .await
                    .map_err(|e| AppError { message: e })?;
            }
            JoinRole::Staff => {
                let staff_repo = SchoolStaffRepo::new(school_db);
                let staff_service = SchoolStaffService::new(&staff_repo);

                let staff = SchoolStaff {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    r#type: self.parse_staff_type(&request.r#type),
                    is_active: true,
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                staff_service
                    .create_school_staff(staff)
                    .await
                    .map_err(|e| AppError { message: e })?;
            }
        }

        Ok(())
    }

    // ----------------------------------------------------------------------
    // Utility helpers
    // ----------------------------------------------------------------------
    async fn generate_registration_number(&self, school: &School) -> Option<String> {
        let year = Utc::now().year();
        let random = rand::random::<u16>() % 10000;
        Some(format!("{}-{}-{:04}", school.username, year, random))
    }

    fn parse_teacher_type(&self, type_str: &str) -> TeacherType {
        match type_str.to_lowercase().as_str() {
            "headteacher" => TeacherType::HeadTeacher,
            "subjectteacher" => TeacherType::SubjectTeacher,
            "deputy" => TeacherType::Deputy,
            _ => TeacherType::Regular,
        }
    }

    fn parse_staff_type(&self, type_str: &str) -> SchoolStaffType {
        match type_str.to_lowercase().as_str() {
            "director" => SchoolStaffType::Director,
            "headofstudies" => SchoolStaffType::HeadOfStudies,
            _ => SchoolStaffType::HeadOfStudies,
        }
    }

    // ----------------------------------------------------------------------
    // Remaining basic methods (reject, cancel, etc.)
    // ----------------------------------------------------------------------
    pub async fn reject_join_request(
        &self,
        respond_request: RespondToJoinRequest,
        rejected_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let request_id = IdType::String(respond_request.request_id.clone());
        let request = self
            .join_request_repo
            .find_by_id(&request_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Join request not found".into(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Join request is not pending".into(),
            });
        }

        let updated_request = self
            .join_request_repo
            .reject_request(&request_id, rejected_by)
            .await?;

        Ok(updated_request)
    }

    pub async fn cancel_join_request(
        &self,
        respond_request: RespondToJoinRequest,
        cancelled_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let request_id = IdType::String(respond_request.request_id.clone());
        let request = self
            .join_request_repo
            .find_by_id(&request_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Join request not found".into(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Only pending requests can be cancelled".into(),
            });
        }

        let updated_request = self
            .join_request_repo
            .cancel_request(&request_id, cancelled_by)
            .await?;

        Ok(updated_request)
    }

    pub async fn get_join_requests(
        &self,
        query: JoinRequestQuery,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let requests = self.join_request_repo.query_requests(query).await?;
        Ok(requests)
    }

    pub async fn get_join_requests_with_relations(
        &self,
        query: JoinRequestQuery,
    ) -> Result<Vec<crate::domain::join_school_request::JoinSchoolRequestWithRelations>, AppError>
    {
        let requests = self.join_request_repo.query_with_relations(&query).await?;
        Ok(requests)
    }

    pub async fn update_request_expiration(
        &self,
        update_expiration: UpdateRequestExpiration,
    ) -> Result<JoinSchoolRequest, AppError> {
        let request_id = IdType::String(update_expiration.request_id.clone());

        let request = self
            .join_request_repo
            .find_by_id(&request_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Join request not found".into(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Can only update expiration for pending requests".into(),
            });
        }

        let updated_request = self
            .join_request_repo
            .update_expiration(&request_id, update_expiration.expires_at)
            .await?;

        Ok(updated_request)
    }

    pub async fn bulk_respond_to_requests(
        &self,
        bulk_respond: BulkRespondRequest,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let requests = self.join_request_repo.bulk_respond(&bulk_respond).await?;
        Ok(requests)
    }

    pub async fn expire_old_requests(&self) -> Result<u64, AppError> {
        let expired_count = self.join_request_repo.expire_old_requests().await?;
        Ok(expired_count)
    }

    pub async fn cleanup_expired_requests(&self, older_than_days: i64) -> Result<u64, AppError> {
        let deleted_count = self
            .join_request_repo
            .cleanup_expired_requests(older_than_days)
            .await?;
        Ok(deleted_count)
    }

    /// Get pending requests for a school
    pub async fn get_pending_requests_for_school(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let requests = self
            .join_request_repo
            .find_by_school_and_status(school_id, JoinStatus::Pending)
            .await?;
        Ok(requests)
    }
}
