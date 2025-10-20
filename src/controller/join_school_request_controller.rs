use actix_web::{web, HttpResponse};
use chrono::{Datelike, Utc};
use mongodb::bson::oid::ObjectId;

use crate::{
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

    /// Create a new join school request
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

        // Parse school_id
        let school_id: IdType = IdType::String(create_request.school_id.clone());
        let school_obj_id = self
            .school_service
            .parse_school_id(&school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        // Check if school exists
        let _school = self
            .school_service
            .get_school_by_id(&school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        // Check for duplicate pending requests
        let duplicate_requests = self
            .join_request_repo
            .find_pending_by_email_and_school(&create_request.email, &school_id)
            .await?;

        if duplicate_requests.is_some() {
            return Err(AppError {
                message: "A pending join request already exists for this email and school"
                    .to_string(),
            });
        }

        // Check if user already exists and is already in the school
        if let Ok(user) = self
            .user_service
            .get_user_by_email(&create_request.email)
            .await
        {
            if let Some(schools) = &user.schools {
                if schools.contains(&school_obj_id) {
                    return Err(AppError {
                        message: "User is already a member of this school".to_string(),
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
            r#type: self.get_default_type_for_role(&create_request.role),
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

    /// Create multiple join requests at once
    pub async fn bulk_create_join_requests(
        &self,
        bulk_request: BulkCreateJoinSchoolRequest,
        sent_by: ObjectId,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let mut requests_to_create = Vec::new();

        for create_request in bulk_request.requests {
            // Validate email
            if let Err(_e) = is_valid_email(&create_request.email) {
                continue; // Skip invalid emails
            }

            // Parse school_id
            let school_id = IdType::String(create_request.school_id.clone());
            let school_obj_id = match self.school_service.parse_school_id(&school_id).await {
                Ok(id) => id,
                Err(_) => continue, // Skip invalid school IDs
            };

            // Check for duplicate pending requests
            if let Ok(Some(_)) = self
                .join_request_repo
                .find_pending_by_email_and_school(&create_request.email, &school_id)
                .await
            {
                continue; // Skip duplicates
            }

            let join_request = JoinSchoolRequest {
                id: None,
                school_id: school_obj_id,
                invited_user_id: None,
                role: create_request.role.clone(),
                email: create_request.email.clone(),
                r#type: self.get_default_type_for_role(&create_request.role),
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
                message: "No valid requests to create".to_string(),
            });
        }

        let created_requests = self
            .join_request_repo
            .bulk_create(requests_to_create)
            .await?;
        Ok(created_requests)
    }

    /// Accept a join request and create the user/role entity
    pub async fn accept_join_request(
        &self,
        respond_request: RespondToJoinRequest,
        accepted_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let request_id = IdType::String(respond_request.request_id.clone());

        // Get the join request
        let request = self
            .join_request_repo
            .find_by_id(&request_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Join request not found".to_string(),
            })?;

        // Validate request status
        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Join request is not pending".to_string(),
            });
        }

        // Get school
        let school_id = IdType::ObjectId(request.school_id);
        let school = self
            .school_service
            .get_school_by_id(&school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        // Find or create user
        let user = match self.user_service.get_user_by_email(&request.email).await {
            Ok(user) => user,
            Err(e) => {
                return Err(AppError {
                    message: format!("Failed to find user: {}", e),
                })
            }
        };

        // Create the role-specific entity (student, teacher, staff)
        self.create_role_entity(&request, &user, &school).await?;

        // Add school to user's schools and set as current
        self.user_service
            .add_school_to_user(&IdType::ObjectId(user.id.unwrap()), &school_id)
            .await
            .map_err(|e| AppError { message: e })?;

        // Update the join request status
        let updated_request = self
            .join_request_repo
            .accept_request(&request_id, user.id.unwrap(), accepted_by)
            .await?;

        Ok(updated_request)
    }

    /// Reject a join request
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
                message: "Join request not found".to_string(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Join request is not pending".to_string(),
            });
        }

        let updated_request = self
            .join_request_repo
            .reject_request(&request_id, rejected_by)
            .await?;

        Ok(updated_request)
    }

    /// Cancel a join request
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
                message: "Join request not found".to_string(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Only pending requests can be cancelled".to_string(),
            });
        }

        let updated_request = self
            .join_request_repo
            .cancel_request(&request_id, cancelled_by)
            .await?;

        Ok(updated_request)
    }

    /// Get join requests with filtering and pagination
    pub async fn get_join_requests(
        &self,
        query: JoinRequestQuery,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let requests = self.join_request_repo.query_requests(query).await?;
        Ok(requests)
    }

    /// Get join requests with relations (school, user, sender)
    pub async fn get_join_requests_with_relations(
        &self,
        query: JoinRequestQuery,
    ) -> Result<Vec<crate::domain::join_school_request::JoinSchoolRequestWithRelations>, AppError>
    {
        let requests = self.join_request_repo.query_with_relations(&query).await?;
        Ok(requests)
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

    /// Update request expiration
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
                message: "Join request not found".to_string(),
            })?;

        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Can only update expiration for pending requests".to_string(),
            });
        }

        let updated_request = self
            .join_request_repo
            .update_expiration(&request_id, update_expiration.expires_at)
            .await?;

        Ok(updated_request)
    }

    /// Bulk respond to multiple requests
    pub async fn bulk_respond_to_requests(
        &self,
        bulk_respond: BulkRespondRequest,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let requests = self.join_request_repo.bulk_respond(&bulk_respond).await?;
        Ok(requests)
    }

    /// Expire old requests (cron job)
    pub async fn expire_old_requests(&self) -> Result<u64, AppError> {
        let expired_count = self.join_request_repo.expire_old_requests().await?;
        Ok(expired_count)
    }

    /// Cleanup expired requests (cron job)
    pub async fn cleanup_expired_requests(&self, older_than_days: i64) -> Result<u64, AppError> {
        let deleted_count = self
            .join_request_repo
            .cleanup_expired_requests(older_than_days)
            .await?;
        Ok(deleted_count)
    }

    // Helper methods

    /// Create role-specific entity (student, teacher, staff)
    async fn create_role_entity(
        &self,
        request: &JoinSchoolRequest,
        user: &User,
        school: &School,
    ) -> Result<(), AppError> {
        let user_id = user.id.unwrap();
        let school_id = request.school_id;

        match request.role {
            JoinRole::Student => {
                let student = Student {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    class_id: None, // Can be assigned later
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

                self.student_service
                    .create_student(student)
                    .await
                    .map_err(|e| AppError { message: e })?;
            }
            JoinRole::Teacher => {
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

                self.teacher_service
                    .create_teacher(teacher)
                    .await
                    .map_err(|e| AppError { message: e })?;
            }
            JoinRole::Staff => {
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

                self.staff_service
                    .create_school_staff(staff)
                    .await
                    .map_err(|e| AppError { message: e })?;
            }
        }

        Ok(())
    }

    /// Generate registration number for student
    async fn generate_registration_number(&self, school: &School) -> Option<String> {
        // Simple implementation - you might want a more sophisticated one
        let year = Utc::now().year();
        let random = rand::random::<u16>() % 10000;
        Some(format!("{}-{}-{:04}", school.username, year, random))
    }

    /// Get default type for role
    fn get_default_type_for_role(&self, role: &JoinRole) -> String {
        match role {
            JoinRole::Student => "Active".to_string(),
            JoinRole::Teacher => "Regular".to_string(),
            JoinRole::Staff => "Staff".to_string(), // or whatever default staff type you have
        }
    }

    /// Parse teacher type from string
    fn parse_teacher_type(&self, type_str: &str) -> TeacherType {
        match type_str.to_lowercase().as_str() {
            "headteacher" => TeacherType::HeadTeacher,
            "subjectteacher" => TeacherType::SubjectTeacher,
            "deputy" => TeacherType::Deputy,
            _ => TeacherType::Regular,
        }
    }

    /// Parse staff type from string
    fn parse_staff_type(&self, type_str: &str) -> SchoolStaffType {
        match type_str.to_lowercase().as_str() {
            "director" => SchoolStaffType::Director,
            "headofstudies" => SchoolStaffType::HeadOfStudies,
            _ => SchoolStaffType::HeadOfStudies, // default
        }
    }
}
