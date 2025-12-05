use std::str::FromStr;

use actix_web::web;
use chrono::{Datelike, Utc};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        class::{Class, ClassLevelType, ClassType},
        class_subject::ClassSubject,
        school::{SchoolAcademicRequest, SchoolAcademicResponse},
    },
    models::id_model::IdType,
    repositories::{
        class_repo::ClassRepo, main_class_repo::MainClassRepo, school_repo::SchoolRepo,
        trade_repo::TradeRepo,
    },
    services::{
        class_service::ClassService, class_subject_service::ClassSubjectService,
        school_service::SchoolService, template_subject_service::TemplateSubjectService,
        trade_service::TradeService,
    },
};

pub struct SchoolController<'a> {
    school_repo: &'a SchoolRepo,
    main_class_repo: &'a MainClassRepo,
    trade_repo: &'a TradeRepo,
    template_subject: &'a TemplateSubjectService,
}

impl<'a> SchoolController<'a> {
    pub fn new(
        school_repo: &'a SchoolRepo,
        main_class_repo: &'a MainClassRepo,
        trade_repo: &'a TradeRepo,
        template_subject: &'a TemplateSubjectService,
    ) -> Self {
        Self {
            school_repo,
            main_class_repo,
            trade_repo,
            template_subject,
        }
    }

    pub async fn setup_school_academics(
        &self,
        school_id: &IdType,
        req: SchoolAcademicRequest,
        state: web::Data<AppState>,
        logged_user: AuthUserDto,
    ) -> Result<SchoolAcademicResponse, String> {
        // Init school service
        let school_service = SchoolService::new(self.school_repo);
        let school = school_service.get_school_by_id(school_id).await?;

        // Validate DB
        let school_db_name = school
            .database_name
            .as_ref()
            .ok_or("School database not configured")?;
        let school_db = state.db.get_db(school_db_name);

        // Local repos (must stay alive)
        let class_repo_school = ClassRepo::new(&school_db);

        let class_subject_service = ClassSubjectService::new(&school_db);

        let class_service_school = ClassService::new(&class_repo_school);

        // Other global services
        let trade_service = TradeService::new(self.trade_repo);

        // Academic year
        let current_year = Utc::now().year();
        let academic_year = format!("{}-{}", current_year, current_year + 1);

        // Get trades
        let sector_ids = req.sector_ids.unwrap_or_default();
        let trade_ids = req.trade_ids.unwrap_or_default();

        let trades = match (!trade_ids.is_empty(), !sector_ids.is_empty()) {
            (true, _) => trade_service.get_trades_by_ids(&trade_ids).await?,
            (false, true) => trade_service.get_trades_by_sector_ids(&sector_ids).await?,
            _ => return Err("No sectors or trades selected".to_string()),
        };

        let creator_id = Some(
            ObjectId::from_str(&logged_user.id)
                .map_err(|e| format!("Failed to convert creator id: {}", e))?,
        );

        let mut created_subjects_count = 0;

        // ---- Step 1: Build list of classes to create ----
        let mut class_trade_pairs = Vec::new();

        for trade in trades {
            let trade_id = trade
                .id
                .ok_or_else(|| format!("Trade id not found for trade '{}'", trade.name))?;
            let trade_id = IdType::ObjectId(trade_id);

            let main_classes = self
                .main_class_repo
                .find_by_trade_id(&trade_id)
                .await
                .map_err(|e| e.message)?;

            for main_class_wrapper in main_classes {
                let main_class = main_class_wrapper.main_class;

                if main_class.disable.unwrap_or(false) {
                    continue;
                }

                let level = main_class.level.unwrap_or(0);

                let class_name = format!(
                    "{:?} {} {} {}",
                    trade.r#type, level, trade.name, academic_year
                );

                let class_username = format!(
                    "{:?}_{}_{}_{}",
                    trade.r#type,
                    level,
                    trade.name.replace(' ', "_"),
                    academic_year.replace('-', "_")
                )
                .to_lowercase();

                let class = Class {
                    id: None,
                    name: class_name,
                    username: class_username,
                    code: None,
                    description: Some(format!("Class for {} - {}", trade.name, academic_year)),
                    school_id: school.id,
                    class_teacher_id: None,
                    r#type: ClassType::School,
                    subject: None,
                    grade_level: Some(level.to_string()),
                    tags: vec!["academic".into(), trade.name.clone()],
                    is_active: true,
                    capacity: Some(30),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    creator_id,
                    main_class_id: main_class.id,
                    trade_id: Some(trade.id.unwrap()),
                    image: None,
                    image_id: None,
                    background_images: None,
                    subclass_ids: Some(vec![]),
                    parent_class_id: None,
                    level_type: Some(ClassLevelType::MainClass),
                };

                class_trade_pairs.push((class, trade.r#type.clone()));
            }
        }

        // No classes
        if class_trade_pairs.is_empty() {
            return Ok(SchoolAcademicResponse {
                success: true,
                created_classes: 0,
                created_subjects: 0,
            });
        }

        // --- Insert classes into school DB ---
        let classes_to_create: Vec<Class> = class_trade_pairs
            .iter()
            .map(|(class, _)| class.clone())
            .collect();

        let created_classes = class_service_school
            .create_many_classes(classes_to_create)
            .await?;
        let created_classes_count = created_classes.len();

        // ---- Step 3: Create Subjects Using TEMPLATE SUBJECTS ----
        for (mut class, trade_type) in class_trade_pairs {
            let Some(main_class_id) = class.main_class_id else {
                continue;
            };

            // map local class.id
            if let Some(created) = created_classes
                .iter()
                .find(|c| c.username == class.username)
            {
                class.id = created.id;
            }

            // TEMPLATE SUBJECTS BY PREREQUISITE (main_class_id)
            let template_subjects = self
                .template_subject
                .find_many_by_prerequisite(&IdType::ObjectId(main_class_id))
                .await
                .map_err(|e| e.message)?;

            let mut class_subjects = Vec::new();

            for t_sub in template_subjects {
                let level = class.name.split_whitespace().nth(1).unwrap_or("0");

                let subject_name = format!(
                    "{} {:?} {} {}",
                    t_sub.name, trade_type, level, academic_year
                );

                class_subjects.push(ClassSubject {
                    id: None,
                    teacher_id: None,
                    school_id: school.id,
                    credits: t_sub.credits,
                    name: subject_name,
                    class_id: class.id,
                    estimated_hours: t_sub.estimated_hours,
                    created_by: creator_id,
                    main_subject_id: None, // removed (template subjects do not use this)
                    category: t_sub.category.clone(),
                    description: Some(t_sub.description.clone()),
                    code: t_sub.code.clone(),
                    topics: t_sub.topics,
                    disable: Some(false),
                    created_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                });
            }

            if !class_subjects.is_empty() {
                let created_subjects = class_subject_service
                    .create_many(class_subjects)
                    .await
                    .map_err(|e| e.message)?;

                created_subjects_count += created_subjects.len();
            }
        }

        // ---- Summary ----
        Ok(SchoolAcademicResponse {
            success: true,
            created_classes: created_classes_count,
            created_subjects: created_subjects_count,
        })
    }
}
