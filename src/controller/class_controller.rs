use chrono::Utc;
use mongodb::bson::oid::ObjectId;

use crate::{
    domain::class::{
        Class, ClassLevelType, ClassStatistics, ClassWithOthers, ClassWithSchool,
        MainClassHierarchy, MainClassWithSubclassCount, MainClassWithSubclasses,
    },
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

    // ===========================
    // NEW SUBCLASS CONTROLLER METHODS
    // ===========================

    /// Get main classes with their subclasses (hierarchy data)
    pub async fn get_main_classes_with_subclasses(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<MainClassWithSubclasses>, AppError> {
        let class_service = ClassService::new(&self.class_repo);

        // Get main classes
        let main_classes = class_service
            .get_main_classes(filter, limit, skip)
            .await
            .map_err(|e| AppError { message: e })?;

        let mut result = Vec::new();

        for main_class in main_classes {
            // Get subclasses for this main class
            let subclasses = class_service
                .get_subclasses(&IdType::from_object_id(main_class.id.unwrap()))
                .await
                .map_err(|e| AppError { message: e })?;

            // Enrich main class with relations
            let main_class_with_others = self.enrich_class_with_relations(main_class).await?;

            result.push(MainClassWithSubclasses {
                main_class: main_class_with_others,
                subclasses: subclasses.into_iter().map(sanitize_class).collect(),
            });
        }

        Ok(result)
    }

    /// Get a specific main class with all its subclasses and full details
    pub async fn get_main_class_hierarchy_with_details(
        &self,
        main_class_id: &IdType,
    ) -> Result<MainClassHierarchy, AppError> {
        let class_service = ClassService::new(&self.class_repo);

        // Get main class with details
        let main_class = self.get_class_by_id_with_others(main_class_id).await?;

        // Verify it's actually a main class
        if main_class.class.level_type != Some(ClassLevelType::MainClass) {
            return Err(AppError {
                message: "Class is not a main class".to_string(),
            });
        }

        // Get subclasses with full details
        let subclasses = class_service
            .get_subclasses_with_others(main_class_id)
            .await
            .map_err(|e| AppError { message: e })?;

        Ok(MainClassHierarchy {
            main_class,
            subclasses,
        })
    }

    /// Get only main classes (without subclasses) with full details
    pub async fn get_main_classes_with_details(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        let class_service = ClassService::new(&self.class_repo);

        // Get main classes
        let main_classes = class_service
            .get_main_classes(filter, limit, skip)
            .await
            .map_err(|e| AppError { message: e })?;

        let mut result = Vec::new();

        for main_class in main_classes {
            let main_class_with_details = self.enrich_class_with_relations(main_class).await?;
            result.push(main_class_with_details);
        }

        Ok(result)
    }

    /// Get only subclasses (without main classes) with full details
    pub async fn get_all_subclasses_with_details(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        // We'll get all classes and filter for subclasses
        let all_classes = self
            .class_repo
            .get_all_classes(filter, limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut subclasses_with_details = Vec::new();

        for class in all_classes {
            if class.level_type == Some(ClassLevelType::SubClass) {
                let subclass_with_details = self.enrich_class_with_relations(class).await?;
                subclasses_with_details.push(subclass_with_details);
            }
        }

        Ok(subclasses_with_details)
    }

    /// Get subclasses by parent class ID with full details
    pub async fn get_subclasses_by_parent_with_details(
        &self,
        parent_class_id: &IdType,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        let class_service = ClassService::new(&self.class_repo);

        let subclasses = class_service
            .get_subclasses_with_others(parent_class_id)
            .await
            .map_err(|e| AppError { message: e })?;

        Ok(subclasses)
    }

    /// Get class statistics (count of main classes, subclasses, etc.)
    pub async fn get_class_statistics(&self) -> Result<ClassStatistics, AppError> {
        let class_service = ClassService::new(&self.class_repo);

        // Get all classes for counting
        let all_classes = self
            .class_repo
            .get_all_classes(None, None, None)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut main_class_count = 0;
        let mut subclass_count = 0;
        let mut active_class_count = 0;
        let mut inactive_class_count = 0;

        for class in all_classes {
            if class.level_type == Some(ClassLevelType::MainClass) {
                main_class_count += 1;
            } else if class.level_type == Some(ClassLevelType::SubClass) {
                subclass_count += 1;
            }

            if class.is_active {
                active_class_count += 1;
            } else {
                inactive_class_count += 1;
            }
        }

        Ok(ClassStatistics {
            total_classes: main_class_count + subclass_count,
            main_classes: main_class_count,
            subclasses: subclass_count,
            active_classes: active_class_count,
            inactive_classes: inactive_class_count,
        })
    }

    /// Search classes by type (main class or subclass)
    pub async fn search_classes_by_level_type(
        &self,
        level_type: ClassLevelType,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        // We'll get all classes and filter by level type
        let all_classes = self
            .class_repo
            .get_all_classes(filter, limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut filtered_classes = Vec::new();

        for class in all_classes {
            if class.level_type == Some(level_type.clone()) {
                let class_with_details = self.enrich_class_with_relations(class).await?;
                filtered_classes.push(class_with_details);
            }
        }

        Ok(filtered_classes)
    }

    /// Get classes that have no subclasses (empty main classes or subclasses)
    pub async fn get_classes_without_subclasses(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<ClassWithOthers>, AppError> {
        let all_classes = self
            .class_repo
            .get_all_classes(filter, limit, skip)
            .await
            .map_err(|e| AppError { message: e.message })?;

        let mut result = Vec::new();

        for class in all_classes {
            let has_subclasses = if class.level_type == Some(ClassLevelType::MainClass) {
                class.subclass_ids.as_ref().map_or(false, |v| !v.is_empty())
            } else {
                false // Subclasses don't have subclasses
            };

            if !has_subclasses {
                let class_with_details = self.enrich_class_with_relations(class).await?;
                result.push(class_with_details);
            }
        }

        Ok(result)
    }

    /// Get classes with the most subclasses (top N main classes)
    pub async fn get_main_classes_by_subclass_count(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<MainClassWithSubclassCount>, AppError> {
        let class_service = ClassService::new(&self.class_repo);

        let main_classes = class_service
            .get_main_classes(None, None, None)
            .await
            .map_err(|e| AppError { message: e })?;

        let mut main_classes_with_count: Vec<MainClassWithSubclassCount> = Vec::new();

        for main_class in main_classes {
            let subclass_count = main_class.subclass_ids.as_ref().map_or(0, |v| v.len());

            let main_class_with_details = self.enrich_class_with_relations(main_class).await?;

            main_classes_with_count.push(MainClassWithSubclassCount {
                main_class: main_class_with_details,
                subclass_count,
            });
        }

        // Sort by subclass count descending
        main_classes_with_count.sort_by(|a, b| b.subclass_count.cmp(&a.subclass_count));

        // Apply limit
        if let Some(limit_val) = limit {
            main_classes_with_count.truncate(limit_val as usize);
        }

        Ok(main_classes_with_count)
    }
}
