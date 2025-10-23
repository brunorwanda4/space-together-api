use crate::{
    domain::class::{Class, ClassWithOthers, ClassWithSchool, UpdateClass},
    errors::AppError,
    models::id_model::IdType,
    repositories::class_repo::ClassRepo,
    services::{
        main_class_service::MainClassService, school_service::SchoolService,
        teacher_service::TeacherService, user_service::UserService,
    },
    utils::class_utils::{sanitize_class, sanitize_classes},
};

pub struct ClassController<'a> {
    pub class_repo: ClassRepo,
    pub school_service: &'a SchoolService<'a>,
    pub user_service: &'a UserService<'a>,
    pub teacher_service: &'a TeacherService<'a>,
    pub main_class_service: &'a MainClassService<'a>,
}

impl<'a> ClassController<'a> {
    pub fn new(
        class_repo: ClassRepo,
        school_service: &'a SchoolService<'a>,
        user_service: &'a UserService<'a>,
        teacher_service: &'a TeacherService<'a>,
        main_class_service: &'a MainClassService<'a>,
    ) -> Self {
        Self {
            class_repo,
            school_service,
            user_service,
            teacher_service,
            main_class_service,
        }
    }

    // ----------------------------------------------------------------------
    // Get all classes with related data from multiple databases
    // ----------------------------------------------------------------------
    pub async fn get_all_school_classes_with_others(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        // First, get classes from school database
        let classes = self
            .class_repo
            .get_all_classes(filter.clone(), limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut classes_with_others = Vec::new();

        for class in classes {
            let class_with_others = self.enrich_class_with_relations(class).await?;
            classes_with_others.push(class_with_others);
        }

        Ok(classes_with_others)
    }

    // ----------------------------------------------------------------------
    // Get class by ID with related data
    // ----------------------------------------------------------------------
    pub async fn get_class_by_id_with_others(
        &self,
        id: &IdType,
    ) -> Result<ClassWithOthers, AppError> {
        let class = self
            .class_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Class not found".into(),
            })?;

        self.enrich_class_with_relations(class).await
    }

    // ----------------------------------------------------------------------
    // Get class by username with related data
    // ----------------------------------------------------------------------
    pub async fn get_class_by_username_with_others(
        &self,
        username: &str,
    ) -> Result<ClassWithOthers, AppError> {
        let class = self
            .class_repo
            .find_by_username(username)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Class not found".into(),
            })?;

        self.enrich_class_with_relations(class).await
    }

    // ----------------------------------------------------------------------
    // Get class by code with related data
    // ----------------------------------------------------------------------
    pub async fn get_class_by_code_with_others(
        &self,
        code: &str,
    ) -> Result<ClassWithOthers, AppError> {
        let class = self
            .class_repo
            .find_by_code(code)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Class not found".into(),
            })?;

        self.enrich_class_with_relations(class).await
    }

    // ----------------------------------------------------------------------
    // Enrich class with related data from different databases
    // ----------------------------------------------------------------------
    async fn enrich_class_with_relations(&self, class: Class) -> Result<ClassWithOthers, AppError> {
        let mut school = None;
        let mut creator = None;
        let mut class_teacher = None;
        let mut main_class = None;

        // Get school from main database
        if let Some(school_id) = class.school_id {
            let school_id_type = IdType::ObjectId(school_id);
            match self.school_service.get_school_by_id(&school_id_type).await {
                Ok(s) => school = Some(s),
                Err(e) => {
                    // Log error but don't fail the whole request
                    eprintln!("Failed to fetch school: {}", e);
                }
            }
        }

        // Get creator (user) from main database
        if let Some(creator_id) = class.creator_id {
            let creator_id_type = IdType::ObjectId(creator_id);
            match self.user_service.get_user_by_id(&creator_id_type).await {
                Ok(u) => creator = Some(u),
                Err(e) => {
                    eprintln!("Failed to fetch creator: {}", e);
                }
            }
        }

        // Get class teacher from school database
        if let Some(teacher_id) = class.class_teacher_id {
            let teacher_id_type = IdType::ObjectId(teacher_id);
            match self
                .teacher_service
                .get_teacher_by_id(&teacher_id_type)
                .await
            {
                Ok(t) => class_teacher = Some(t),
                Err(e) => {
                    eprintln!("Failed to fetch class teacher: {}", e);
                }
            }
        }

        // Get main class from main database
        if let Some(main_class_id) = class.main_class_id {
            let main_class_id_type = IdType::ObjectId(main_class_id);
            match self.main_class_service.get_by_id(&main_class_id_type).await {
                Ok(mc) => main_class = Some(mc),
                Err(e) => {
                    eprintln!("Failed to fetch main class: {}", e);
                }
            }
        }

        Ok(ClassWithOthers {
            class: sanitize_class(class),
            school,
            creator,
            class_teacher,
            main_class,
        })
    }

    // ----------------------------------------------------------------------
    // Get classes with school information only
    // ----------------------------------------------------------------------
    pub async fn get_all_classes_with_school(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithSchool>, AppError> {
        let classes = self
            .class_repo
            .get_all_classes(filter.clone(), limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut classes_with_school = Vec::new();

        for class in classes {
            let mut school = None;

            // Get school from main database
            if let Some(school_id) = class.school_id {
                let school_id_type = IdType::ObjectId(school_id);
                if let Ok(s) = self.school_service.get_school_by_id(&school_id_type).await {
                    school = Some(s);
                }
            }

            classes_with_school.push(ClassWithSchool {
                class: sanitize_class(class),
                school,
            });
        }

        Ok(classes_with_school)
    }

    // ----------------------------------------------------------------------
    // Basic class operations (delegate to repo)
    // ----------------------------------------------------------------------
    pub async fn get_all_classes(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Class>, AppError> {
        let classes = self
            .class_repo
            .get_all_classes(filter, limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;
        Ok(sanitize_classes(classes))
    }

    pub async fn get_class_by_id(&self, id: &IdType) -> Result<Class, AppError> {
        let class = self
            .class_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError { message: e.message })?
            .ok_or_else(|| AppError {
                message: "Class not found".into(),
            })?;
        Ok(sanitize_class(class))
    }

    pub async fn create_class(&self, class: Class) -> Result<Class, AppError> {
        let created_class = self
            .class_repo
            .insert_class(&class)
            .await
            .map_err(|e| AppError { message: e.message })?;
        Ok(sanitize_class(created_class))
    }

    pub async fn update_class(
        &self,
        id: &IdType,
        updated_data: &UpdateClass,
    ) -> Result<Class, AppError> {
        let updated_class = self
            .class_repo
            .update_class(id, updated_data)
            .await
            .map_err(|e| AppError { message: e.message })?;
        Ok(sanitize_class(updated_class))
    }

    pub async fn delete_class(&self, id: &IdType) -> Result<(), AppError> {
        self.class_repo
            .delete_class(id)
            .await
            .map_err(|e| AppError { message: e.message })
    }

    // ----------------------------------------------------------------------
    // Batch operations for better performance
    // ----------------------------------------------------------------------
    pub async fn get_classes_with_others_batch(
        &self,
        class_ids: Vec<IdType>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        let mut classes = Vec::new();

        for id in class_ids {
            if let Ok(class) = self.get_class_by_id_with_others(&id).await {
                classes.push(class);
            }
        }

        Ok(classes)
    }

    // ----------------------------------------------------------------------
    // Filter classes with relations
    // ----------------------------------------------------------------------
    pub async fn get_classes_by_school_with_relations(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        let classes = self
            .class_repo
            .find_by_school_id(school_id)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut classes_with_others = Vec::new();
        for class in classes {
            let class_with_others = self.enrich_class_with_relations(class).await?;
            classes_with_others.push(class_with_others);
        }

        Ok(classes_with_others)
    }
}
