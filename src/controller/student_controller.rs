use mongodb::bson::Document;

use crate::{
    controller::join_school_request_controller::JoinSchoolRequestController,
    domain::{
        class::Class,
        common_details::Paginated,
        school::School,
        student::{Student, StudentWithRelations},
        user::User,
    },
    errors::AppError,
    models::id_model::IdType,
    services::{
        class_service::ClassService, school_service::SchoolService,
        student_service::StudentService, user_service::UserService,
    },
};

pub struct StudentController<'a> {
    pub student_service: &'a StudentService,
    pub user_service: &'a UserService<'a>,
    pub school_service: &'a SchoolService<'a>,
    pub class_service: &'a ClassService<'a>,
    pub join_school_request: &'a JoinSchoolRequestController<'a>,
}

impl<'a> StudentController<'a> {
    pub fn new(
        student_service: &'a StudentService,
        user_service: &'a UserService<'a>,
        school_service: &'a SchoolService<'a>,
        class_service: &'a ClassService<'a>,
        join_school_request: &'a JoinSchoolRequestController<'a>,
    ) -> Self {
        Self {
            student_service,
            user_service,
            school_service,
            class_service,
            join_school_request,
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
        extra_match: Option<Document>,
    ) -> Result<Paginated<StudentWithRelations>, AppError> {
        let paginated = self
            .student_service
            .get_all(filter, limit, skip, extra_match)
            .await?;
        let students = paginated.data;

        let mut results = Vec::new();

        for student in students {
            let enriched = self.enrich_student_with_relations(student).await?;
            results.push(enriched);
        }

        Ok(Paginated {
            data: results,
            total: paginated.total,
            total_pages: paginated.total_pages,
            current_page: paginated.current_page,
        })
    }

    // ----------------------------------------------------------------------
    // Get student by ID with related data
    // ----------------------------------------------------------------------
    pub async fn get_student_by_id_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<StudentWithRelations, AppError> {
        let student = self.student_service.find_one(id, extra_match).await?;

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
