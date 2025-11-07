use mongodb::bson::oid::ObjectId;

use crate::{
    controller::join_school_request_controller::JoinSchoolRequestController,
    domain::{
        class::Class,
        join_school_request::{CreateJoinSchoolRequest, JoinRole},
        school::School,
        student::{Student, StudentWithRelations},
        user::User,
    },
    errors::AppError,
    models::id_model::IdType,
    repositories::student_repo::StudentRepo,
    services::{
        class_service::ClassService, school_service::SchoolService,
        student_service::StudentService, user_service::UserService,
    },
};

pub struct StudentController<'a> {
    pub student_repo: &'a StudentRepo,
    pub student_service: &'a StudentService<'a>,
    pub user_service: &'a UserService<'a>,
    pub school_service: &'a SchoolService<'a>,
    pub class_service: &'a ClassService<'a>,
    pub join_school_request: &'a JoinSchoolRequestController<'a>,
}

impl<'a> StudentController<'a> {
    pub fn new(
        student_repo: &'a StudentRepo,
        student_service: &'a StudentService<'a>,
        user_service: &'a UserService<'a>,
        school_service: &'a SchoolService<'a>,
        class_service: &'a ClassService<'a>,
        join_school_request: &'a JoinSchoolRequestController<'a>,
    ) -> Self {
        Self {
            student_repo,
            student_service,
            user_service,
            school_service,
            class_service,
            join_school_request,
        }
    }

    pub async fn create_student(
        &self,
        new_student: Student,
        sent_by: ObjectId,
    ) -> Result<Student, String> {
        // Ensure the student has a school_id
        let school_id = match new_student.school_id {
            Some(id) => id,
            None => return Err("Missing school_id".to_string()),
        };

        // Optional: Ensure the student has a class_id if your logic requires it
        let class_id = new_student.class_id;

        // Build a join school request
        let create_request = CreateJoinSchoolRequest {
            school_id: school_id.to_string(),
            class_id: class_id.map(|id| id.to_string()),
            message: Some("Join School request".to_string()),
            r#type: "Student".to_string(),
            role: JoinRole::Student,
            email: new_student.email.clone(),
            sent_by: sent_by.to_hex(),
        };

        // Try to create the join school request before creating the student
        match self
            .join_school_request
            .create_join_request(create_request, sent_by)
            .await
        {
            Ok(_) => self.student_service.create_student(new_student).await,
            Err(_) => self.student_service.create_student(new_student).await,
        }
    }

    // ----------------------------------------------------------------------
    // Get all students with related data
    // ----------------------------------------------------------------------
    pub async fn get_all_students_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<StudentWithRelations>, AppError> {
        let students = self
            .student_repo
            .get_all_students(filter.clone(), limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut results = Vec::new();

        for student in students {
            let enriched = self.enrich_student_with_relations(student).await?;
            results.push(enriched);
        }

        Ok(results)
    }

    // ----------------------------------------------------------------------
    // Get student by ID with related data
    // ----------------------------------------------------------------------
    pub async fn get_student_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<StudentWithRelations, AppError> {
        let student = self
            .student_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Student not found".into(),
            })?;

        self.enrich_student_with_relations(student).await
    }

    // ----------------------------------------------------------------------
    // Enrich student with related data (user, school, class)
    // ----------------------------------------------------------------------
    async fn enrich_student_with_relations(
        &self,
        student: Student,
    ) -> Result<StudentWithRelations, AppError> {
        let mut user: Option<User> = None;
        let mut creator: Option<User> = None;
        let mut school: Option<School> = None;
        let mut class: Option<Class> = None;

        // ✅ Fetch user, but skip if error
        if let Some(user_id) = student.user_id {
            let id = IdType::ObjectId(user_id);
            user = match self.user_service.get_user_by_id(&id).await {
                Ok(u) => Some(u),
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch user ({}): {}", user_id, e);
                    None
                }
            };
        }

        // ✅ Fetch creator, but skip if error
        if let Some(creator_id) = student.creator_id {
            let id = IdType::ObjectId(creator_id);
            creator = match self.user_service.get_user_by_id(&id).await {
                Ok(u) => Some(u),
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch creator ({}): {}", creator_id, e);
                    None
                }
            };
        }

        // ✅ Fetch school, but skip if error
        if let Some(school_id) = student.school_id {
            let id = IdType::ObjectId(school_id);
            school = match self.school_service.get_school_by_id(&id).await {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch school ({}): {}", school_id, e);
                    None
                }
            };
        }

        // ✅ Fetch class, but skip if error
        if let Some(class_id) = student.class_id {
            let id = IdType::ObjectId(class_id);
            class = match self.class_service.get_class_by_id(&id).await {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch class ({}): {}", class_id, e);
                    None
                }
            };
        }

        Ok(StudentWithRelations {
            student,
            user,
            school,
            class,
            creator,
        })
    }
}
