use crate::{
    domain::{
        common_details::Gender,
        teacher::{
            BulkTeacherTags, BulkUpdateTeacherActive, Teacher, TeacherType, TeacherWithRelations,
            UpdateTeacher,
        },
        user::User,
    },
    errors::AppError,
    models::id_model::IdType,
    repositories::teacher_repo::TeacherRepo,
    services::{
        class_service::ClassService, school_service::SchoolService,
        subject_service::SubjectService, teacher_service::TeacherService,
        user_service::UserService,
    },
};

pub struct TeacherController<'a> {
    pub teacher_repo: &'a TeacherRepo,
    pub school_service: &'a SchoolService<'a>,
    pub user_service: &'a UserService<'a>,
    pub teacher_service: &'a TeacherService<'a>,
    pub subject_service: &'a SubjectService<'a>,
    pub class_service: &'a ClassService<'a>,
}

impl<'a> TeacherController<'a> {
    pub fn new(
        teacher_repo: &'a TeacherRepo,
        school_service: &'a SchoolService<'a>,
        user_service: &'a UserService<'a>,
        teacher_service: &'a TeacherService<'a>,
        subject_service: &'a SubjectService<'a>,
        class_service: &'a ClassService<'a>,
    ) -> Self {
        Self {
            teacher_repo,
            school_service,
            user_service,
            teacher_service,
            subject_service,
            class_service,
        }
    }

    // ----------------------------------------------------------------------
    // Get all teachers with related data from multiple databases
    // ----------------------------------------------------------------------
    pub async fn get_all_teachers_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        // First, get teachers from school database
        let teachers = self
            .teacher_repo
            .get_all_teachers(filter.clone(), limit, skip, None)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut teachers_with_relations = Vec::new();

        for teacher in teachers {
            let teacher_with_relations = self.enrich_teacher_with_relations(teacher).await?;
            teachers_with_relations.push(teacher_with_relations);
        }

        Ok(teachers_with_relations)
    }

    // ----------------------------------------------------------------------
    // Get teacher by ID with related data
    // ----------------------------------------------------------------------
    pub async fn get_teacher_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<TeacherWithRelations, AppError> {
        let teacher = self
            .teacher_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Teacher not found".into(),
            })?;

        self.enrich_teacher_with_relations(teacher).await
    }

    // ----------------------------------------------------------------------
    // Get teacher by user ID with related data
    // ----------------------------------------------------------------------
    pub async fn get_teacher_by_user_id_with_relations(
        &self,
        user_id: &IdType,
    ) -> Result<TeacherWithRelations, AppError> {
        let teacher = self
            .teacher_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Teacher not found for this user".into(),
            })?;

        self.enrich_teacher_with_relations(teacher).await
    }

    // ----------------------------------------------------------------------
    // Get teacher by email with related data
    // ----------------------------------------------------------------------
    pub async fn get_teacher_by_email_with_relations(
        &self,
        email: &str,
    ) -> Result<TeacherWithRelations, AppError> {
        let teacher = self
            .teacher_repo
            .find_by_email(email)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Teacher not found".into(),
            })?;

        self.enrich_teacher_with_relations(teacher).await
    }

    // ----------------------------------------------------------------------
    // Get teachers by school ID with relations
    // ----------------------------------------------------------------------
    pub async fn get_teachers_by_school_id_with_relations(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        let teachers = self
            .teacher_repo
            .find_by_school_id(school_id)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut teachers_with_relations = Vec::new();
        for teacher in teachers {
            let teacher_with_relations = self.enrich_teacher_with_relations(teacher).await?;
            teachers_with_relations.push(teacher_with_relations);
        }

        Ok(teachers_with_relations)
    }

    // ----------------------------------------------------------------------
    // Get teachers by creator ID with relations
    // ----------------------------------------------------------------------
    pub async fn get_teachers_by_creator_id_with_relations(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        let teachers = self
            .teacher_repo
            .find_by_creator_id(creator_id)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut teachers_with_relations = Vec::new();
        for teacher in teachers {
            let teacher_with_relations = self.enrich_teacher_with_relations(teacher).await?;
            teachers_with_relations.push(teacher_with_relations);
        }

        Ok(teachers_with_relations)
    }

    // ----------------------------------------------------------------------
    // Enrich teacher with related data from different databases
    // ----------------------------------------------------------------------
    async fn enrich_teacher_with_relations(
        &self,
        teacher: Teacher,
    ) -> Result<TeacherWithRelations, AppError> {
        let mut school = None;
        let mut creator: Option<User> = None;
        let mut user = None;
        let mut classes = None;
        let mut subjects = None;

        // ✅ Get school from main database
        if let Some(school_id) = teacher.school_id {
            let school_id_type = IdType::ObjectId(school_id);
            match self.school_service.get_school_by_id(&school_id_type).await {
                Ok(s) => school = Some(s),
                Err(e) => {
                    return Err(AppError {
                        message: format!("Failed to fetch school: {}", e),
                    });
                }
            }
        }

        // ✅ Get creator (user) from main database
        if let Some(creator_id) = teacher.creator_id {
            let creator_id_type = IdType::ObjectId(creator_id);
            match self.user_service.get_user_by_id(&creator_id_type).await {
                Ok(u) => creator = Some(u),
                Err(e) => {
                    return Err(AppError {
                        message: format!("Failed to fetch creator: {}", e),
                    });
                }
            }
        }

        // ✅ Get user (account) from main database
        if let Some(user_id) = teacher.user_id {
            let user_id_type = IdType::ObjectId(user_id);
            match self.user_service.get_user_by_id(&user_id_type).await {
                Ok(u) => user = Some(u),
                Err(e) => {
                    return Err(AppError {
                        message: format!("Failed to fetch user: {}", e),
                    });
                }
            }
        }

        // ✅ Fix: use `.into_iter()` to convert Vec into iterator
        if let Some(ref ids) = teacher.subject_ids {
            // Convert subject ObjectIds to IdType
            let subject_id_types: Vec<IdType> =
                ids.iter().map(|id| IdType::ObjectId(*id)).collect();

            // Fetch subjects by IDs
            match self
                .subject_service
                .get_subjects_by_ids(subject_id_types)
                .await
            {
                Ok(subs) => {
                    // Remove duplicates by subject _id
                    let mut unique_subjects = Vec::new();
                    let mut seen_ids = std::collections::HashSet::new();

                    for s in subs {
                        if let Some(id) = s.id {
                            if seen_ids.insert(id) {
                                unique_subjects.push(s);
                            }
                        }
                    }

                    subjects = Some(unique_subjects);
                }
                Err(e) => {
                    return Err(AppError {
                        message: format!("Failed to fetch subjects: {}", e),
                    });
                }
            }
        } else {
            // ✅ Handle missing teacher.id safely
            let teacher_id = match teacher.id {
                Some(id) => IdType::ObjectId(id),
                None => {
                    return Err(AppError {
                        message: "Teacher ID is missing".to_string(),
                    });
                }
            };

            // Fetch subjects taught by this teacher
            let teacher_subjects = self
                .subject_service
                .get_subjects_by_class_teacher_id(&teacher_id)
                .await
                .map_err(|e| AppError { message: e })?;

            // Remove duplicates by _id
            let mut unique_subjects = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();

            for s in teacher_subjects {
                if let Some(id) = s.id {
                    if seen_ids.insert(id) {
                        unique_subjects.push(s);
                    }
                }
            }

            subjects = Some(unique_subjects);
        }

        // ✅ Fix: same issue — Vec is not iterator, use .into_iter()
        if let Some(ref class_ids) = teacher.class_ids {
            let teacher_classes = self
                .class_service
                .get_many_classes_by_ids(class_ids.clone())
                .await
                .map_err(|e| AppError { message: e })?;

            classes = Some(teacher_classes);
        }

        Ok(TeacherWithRelations {
            teacher,
            user,
            school,
            classes,
            subjects,
            creator,
        })
    }

    // ----------------------------------------------------------------------
    // Specialized queries with relations
    // ----------------------------------------------------------------------
    pub async fn get_teachers_by_type_with_relations(
        &self,
        teacher_type: TeacherType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        let teachers = self
            .teacher_repo
            .find_by_type(teacher_type)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut teachers_with_relations = Vec::new();
        for teacher in teachers {
            let teacher_with_relations = self.enrich_teacher_with_relations(teacher).await?;
            teachers_with_relations.push(teacher_with_relations);
        }

        Ok(teachers_with_relations)
    }

    pub async fn get_teachers_by_school_and_type_with_relations(
        &self,
        school_id: &IdType,
        teacher_type: TeacherType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        let teachers = self
            .teacher_repo
            .find_by_school_and_type(school_id, teacher_type)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut teachers_with_relations = Vec::new();
        for teacher in teachers {
            let teacher_with_relations = self.enrich_teacher_with_relations(teacher).await?;
            teachers_with_relations.push(teacher_with_relations);
        }

        Ok(teachers_with_relations)
    }

    // ----------------------------------------------------------------------
    // Count operations
    // ----------------------------------------------------------------------
    pub async fn count_teachers_by_school_id(
        &self,
        school_id: &IdType,
        gender: Option<Gender>,
        teacher_type: Option<TeacherType>,
    ) -> Result<u64, AppError> {
        let count = self
            .teacher_service
            .count_teachers_by_school_id(school_id, gender, teacher_type)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(count)
    }

    pub async fn count_teachers_by_type(&self, teacher_type: TeacherType) -> Result<u64, AppError> {
        let count = self
            .teacher_service
            .count_teachers_by_type(teacher_type)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(count)
    }

    // ----------------------------------------------------------------------
    // Bulk operations
    // ----------------------------------------------------------------------
    pub async fn create_many_teachers(
        &self,
        teachers: Vec<Teacher>,
    ) -> Result<Vec<Teacher>, AppError> {
        let created_teachers = self
            .teacher_service
            .create_many_teachers(teachers)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(created_teachers)
    }

    pub async fn update_many_teachers(
        &self,
        updates: Vec<(IdType, UpdateTeacher)>,
    ) -> Result<Vec<Teacher>, AppError> {
        let updated_teachers = self
            .teacher_service
            .update_many_teachers(updates)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(updated_teachers)
    }

    pub async fn bulk_update_active(
        &self,
        request: &BulkUpdateTeacherActive,
    ) -> Result<Vec<Teacher>, AppError> {
        let updated_teachers = self
            .teacher_service
            .bulk_update_active(request)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(updated_teachers)
    }

    pub async fn bulk_add_tags(&self, request: &BulkTeacherTags) -> Result<Vec<Teacher>, AppError> {
        let updated_teachers = self
            .teacher_service
            .bulk_add_tags(request)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(updated_teachers)
    }

    pub async fn bulk_remove_tags(
        &self,
        request: &BulkTeacherTags,
    ) -> Result<Vec<Teacher>, AppError> {
        let updated_teachers = self
            .teacher_service
            .bulk_remove_tags(request)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(updated_teachers)
    }

    // ----------------------------------------------------------------------
    // Utility methods
    // ----------------------------------------------------------------------
    pub async fn is_user_teacher_of_school(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<bool, AppError> {
        let result = self
            .teacher_service
            .is_user_teacher_of_school(user_id, school_id)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(result)
    }

    pub async fn get_school_teacher_statistics(
        &self,
        school_id: &IdType,
    ) -> Result<std::collections::HashMap<TeacherType, u64>, AppError> {
        let stats = self
            .teacher_service
            .get_school_teacher_statistics(school_id)
            .await
            .map_err(|e| AppError { message: e })?;
        Ok(stats)
    }

    // ----------------------------------------------------------------------
    // Teacher type specific methods with relations
    // ----------------------------------------------------------------------
    pub async fn get_school_head_teachers_with_relations(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        self.get_teachers_by_school_and_type_with_relations(school_id, TeacherType::HeadTeacher)
            .await
    }

    pub async fn get_school_subject_teachers_with_relations(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        self.get_teachers_by_school_and_type_with_relations(school_id, TeacherType::SubjectTeacher)
            .await
    }

    pub async fn get_school_deputy_teachers_with_relations(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        self.get_teachers_by_school_and_type_with_relations(school_id, TeacherType::Deputy)
            .await
    }

    pub async fn get_school_regular_teachers_with_relations(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<TeacherWithRelations>, AppError> {
        self.get_teachers_by_school_and_type_with_relations(school_id, TeacherType::Regular)
            .await
    }
}
