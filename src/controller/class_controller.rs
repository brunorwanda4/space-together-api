use chrono::Utc;
use mongodb::bson::oid::ObjectId;

use crate::{
    domain::class::{Class, ClassLevelType, ClassWithOthers, ClassWithSchool},
    errors::AppError,
    helpers::object_id_helpers::parse_object_id,
    models::id_model::IdType,
    repositories::class_repo::ClassRepo,
    services::{
        class_service::ClassService, main_class_service::MainClassService,
        school_service::SchoolService, teacher_service::TeacherService,
        trade_service::TradeService, user_service::UserService,
    },
    utils::{
        class_utils::sanitize_class, school_utils::sanitize_school, user_utils::sanitize_user,
    },
};

pub struct ClassController<'a> {
    pub class_repo: ClassRepo,
    pub school_service: &'a SchoolService<'a>,
    pub user_service: &'a UserService<'a>,
    pub teacher_service: &'a TeacherService<'a>,
    pub main_class_service: &'a MainClassService<'a>,
    pub trade_service: &'a TradeService<'a>,
}

impl<'a> ClassController<'a> {
    pub fn new(
        class_repo: ClassRepo,
        school_service: &'a SchoolService<'a>,
        user_service: &'a UserService<'a>,
        teacher_service: &'a TeacherService<'a>,
        main_class_service: &'a MainClassService<'a>,
        trade_service: &'a TradeService<'a>,
    ) -> Self {
        Self {
            class_repo,
            school_service,
            user_service,
            teacher_service,
            main_class_service,
            trade_service,
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
        let mut trade = None;

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

        if let Some(trade_id) = class.trade_id {
            let trade_id_type = IdType::ObjectId(trade_id);
            match self.trade_service.get_trade_by_id(&trade_id_type).await {
                Ok(s) => trade = Some(s),
                Err(e) => {
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
            school: school.map(sanitize_school),
            creator: creator.map(sanitize_user),
            class_teacher,
            main_class,
            trade,
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

    pub async fn add_or_update_class_teacher(
        &self,
        class_id: &IdType,
        teacher_id: &IdType,
    ) -> Result<Class, String> {
        // Step 1: get teacher
        let teacher = self.teacher_service.get_teacher_by_id(teacher_id).await?;

        // Step 2: ensure teacher has this class_id
        let class_obj_id = parse_object_id(class_id)?;
        let mut class_ids = teacher.class_ids.unwrap_or_default();

        // Add class if missing
        if !class_ids.contains(&class_obj_id) {
            class_ids.push(class_obj_id);

            // ✅ use your existing service to add class to teacher
            self.teacher_service
                .add_classes_to_teacher(teacher_id, class_ids)
                .await?;
        }

        // Step 3: add or update teacher in class
        let updated_class = self
            .class_repo
            .add_or_update_class_teacher(class_id, teacher_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(updated_class)
    }

    /// Create multiple sub-classes (e.g., Primary 1 A, Primary 1 B, etc.) under a given main class
    pub async fn create_many_sub_class_by_class_id(
        &self,
        main_class_id: &IdType,
        num_sub_classes: u8,
        creator_id: ObjectId,
    ) -> Result<Vec<Class>, String> {
        if !(2..=12).contains(&num_sub_classes) {
            return Err("Count must be a number between 2 and 12".to_string());
        };

        let class_service = ClassService::new(&self.class_repo);
        // 1️⃣ Get the main class first
        let main_class = class_service.get_class_by_id(main_class_id).await?;
        let main_class_id_obj = parse_object_id(main_class_id)?;
        // 2️⃣ Prepare subclass list
        let mut subclasses = Vec::new();

        for i in 0..num_sub_classes {
            // Generate subclass name like "Primary 1 A", "Primary 1 B", etc.
            // Using letters A, B, C... (you can also switch to numbers if you prefer)
            let letter = ((b'A' + i) as char).to_string();

            let subclass_name = format!("{} {}", main_class.name, letter);
            let subclass_username = format!("{}_{}", main_class.username, letter.to_lowercase());

            let now = Utc::now();

            // Build subclass object
            let subclass = Class {
                id: None,
                name: subclass_name,
                username: subclass_username,
                code: None,
                school_id: main_class.school_id,
                creator_id: Some(creator_id),
                class_teacher_id: None, // can be assigned later
                r#type: main_class.r#type.clone(),
                level_type: Some(ClassLevelType::SubClass),
                parent_class_id: Some(main_class_id_obj),
                subclass_ids: None,
                main_class_id: main_class.main_class_id,
                trade_id: main_class.trade_id,
                is_active: true,
                image_id: None,
                image: None,
                background_images: None,
                description: Some(format!("Sub class of {}", main_class.name)),
                capacity: None,
                subject: None,
                grade_level: main_class.grade_level.clone(),
                tags: vec!["subclass".to_string()],
                created_at: now,
                updated_at: now,
            };

            subclasses.push(subclass);
        }

        // 3️⃣ Call service to add subclasses
        let created_subclasses = class_service
            .add_multiple_subclasses(main_class_id, subclasses)
            .await?;

        Ok(created_subclasses)
    }
}
