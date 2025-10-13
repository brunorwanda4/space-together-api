use actix_web::web;
use chrono::{Datelike, Utc};

use crate::{
    config::state::AppState,
    domain::{
        class::{Class, ClassType},
        school::{SchoolAcademicRequest, SchoolAcademicResponse},
        subject::Subject,
    },
    models::id_model::IdType,
    repositories::{
        class_repo::ClassRepo, main_class_repo::MainClassRepo, school_repo::SchoolRepo,
        subject_repo::SubjectRepo, subjects::main_subject_repo::MainSubjectRepo,
        trade_repo::TradeRepo,
    },
    services::{
        class_service::ClassService, school_service::SchoolService,
        subject_service::SubjectService, subjects::main_subject_service::MainSubjectService,
        trade_service::TradeService,
    },
};

pub struct SchoolController<'a> {
    school_repo: &'a SchoolRepo,
    main_class_repo: &'a MainClassRepo,
    main_subject_repo: &'a MainSubjectRepo,
    trade_repo: &'a TradeRepo,
}

impl<'a> SchoolController<'a> {
    pub fn new(
        school_repo: &'a SchoolRepo,
        main_class_repo: &'a MainClassRepo,
        main_subject_repo: &'a MainSubjectRepo,
        trade_repo: &'a TradeRepo,
    ) -> Self {
        Self {
            school_repo,
            main_class_repo,
            main_subject_repo,
            trade_repo,
        }
    }

    pub async fn setup_school_academics(
        &self,
        school_id: &IdType,
        req: SchoolAcademicRequest,
        state: web::Data<AppState>,
    ) -> Result<SchoolAcademicResponse, String> {
        // Initialize services
        let school_service = SchoolService::new(self.school_repo);
        let school = school_service.get_school_by_id(school_id).await?;

        // Validate database setup
        let school_db_name = school
            .database_name
            .as_ref()
            .ok_or("School database not configured")?;
        let school_db = state.db.get_db(school_db_name);

        // ✅ FIXED: keep repos alive
        let class_repo_school = ClassRepo::new(&school_db);
        let subject_repo_school = SubjectRepo::new(&school_db);

        let class_service_school = ClassService::new(&class_repo_school);
        let subject_service_school = SubjectService::new(&subject_repo_school);

        // Global repositories
        let main_subject_service = MainSubjectService::new(self.main_subject_repo);
        let trade_service = TradeService::new(self.trade_repo);

        // Academic year
        let current_year = Utc::now().year();
        let academic_year = format!("{}-{}", current_year, current_year + 1);

        // Get sectors/trades
        let sector_ids = req.sector_ids.unwrap_or_default();
        let trade_ids = req.trade_ids.unwrap_or_default();

        let trades = match (!trade_ids.is_empty(), !sector_ids.is_empty()) {
            (true, _) => trade_service.get_trades_by_ids(&trade_ids).await?,
            (false, true) => trade_service.get_trades_by_sector_ids(&sector_ids).await?,
            (false, false) => return Err("No sectors or trades selected".to_string()),
        };

        let mut created_subjects_count = 0;

        // ---- Step 1: Create Classes ----
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
                    "{:?} {} {} {} {}",
                    trade.r#type, level, trade.name, school.name, academic_year
                );

                let class_username = format!(
                    "{:?}_{}_{}_{}_{}",
                    trade.r#type,
                    level,
                    trade.name.replace(' ', "_"),
                    school.name.replace(' ', "_"),
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
                    creator_id: school.creator_id,
                    main_class_id: main_class.id,
                };

                class_trade_pairs.push((class, trade.r#type.clone()));
            }
        }

        if class_trade_pairs.is_empty() {
            return Ok(SchoolAcademicResponse {
                success: true,
                created_classes: 0,
                created_subjects: 0,
            });
        }

        // ✅ only create counter here to avoid unused assignment warning
        let created_classes_count;

        // Separate classes for DB insertion
        let classes_to_create: Vec<Class> = class_trade_pairs
            .iter()
            .map(|(class, _)| class.clone())
            .collect();

        let created_classes = class_service_school
            .create_many_classes(classes_to_create)
            .await?;
        created_classes_count = created_classes.len();

        // ---- Step 3: Create Subjects for Each Class ----
        for (mut class, trade_type) in class_trade_pairs {
            let Some(main_class_id) = class.main_class_id else {
                continue;
            };

            // Replace local ID if available
            if let Some(created) = created_classes
                .iter()
                .find(|c| c.username == class.username)
            {
                class.id = created.id;
            }

            let main_subjects = main_subject_service
                .get_subjects_by_main_class_id(&IdType::ObjectId(main_class_id))
                .await?;

            let mut class_subjects = Vec::new();

            for main_subject in main_subjects {
                if !main_subject.is_active {
                    continue;
                }

                let level = class.name.split_whitespace().nth(1).unwrap_or("0");

                let subject_name = format!(
                    "{} {:?} {} {} {}",
                    main_subject.name, trade_type, level, school.name, academic_year
                );

                let subject_username = format!(
                    "{}_{:?}_{}_{}_{}",
                    main_subject.name.replace(' ', "_"),
                    trade_type,
                    level,
                    school.name.replace(' ', "_"),
                    academic_year.replace('-', "_")
                )
                .to_lowercase();

                class_subjects.push(Subject {
                    id: None,
                    name: subject_name,
                    username: subject_username,
                    class_id: class.id,
                    creator_id: school.creator_id,
                    class_teacher_id: None,
                    main_subject_id: main_subject.id,
                    subject_type: main_subject.category.clone(),
                    description: main_subject.description.clone(),
                    code: None,
                    is_active: true,
                    tags: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }

            if !class_subjects.is_empty() {
                let created_subjects = subject_service_school
                    .create_many_subjects_for_class(
                        &IdType::ObjectId(class.id.unwrap()),
                        class_subjects,
                    )
                    .await?;
                created_subjects_count += created_subjects.len();
            }
        }

        // ---- Step 4: Return Summary ----
        Ok(SchoolAcademicResponse {
            success: true,
            created_classes: created_classes_count,
            created_subjects: created_subjects_count,
        })
    }
}
