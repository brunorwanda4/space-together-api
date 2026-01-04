use std::str::FromStr;

use actix_web::web;
use chrono::{Datelike, Utc};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        class::{Class, ClassLevelType, ClassSettings, ClassType},
        class_subject::ClassSubject,
        common_details::Paginated,
        school::{School, SchoolAcademicRequest, SchoolAcademicResponse, SchoolPartial},
    },
    errors::AppError,
    mappers::school_mapper::to_school_school_token,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::{base_repo::BaseRepository, class_repo::ClassRepo},
    services::{
        class_service::ClassService, class_subject_service::ClassSubjectService,
        cloudinary_service::CloudinaryService, main_class_service::MainClassService,
        template_subject_service::TemplateSubjectService, trade_service::TradeService,
    },
    utils::{
        code::generate_code,
        mongo_utils::extract_valid_fields,
        names::{is_valid_name, is_valid_username},
        school_token::{create_school_token, verify_school_token},
    },
};

pub struct SchoolService {
    pub collection: Collection<School>,
}

impl SchoolService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<School>("schools"),
        }
    }

    // =========================
    // INDEXES
    // =========================
    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", false),
            IndexDef::single("username", true),
            IndexDef::single("code", true),
            IndexDef::single("creator_id", false),
            IndexDef::single("is_active", false),
            IndexDef::compound(vec![("username", 1), ("is_active", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: School) -> Result<School, AppError> {
        self.ensure_indexes().await?;
        is_valid_username(&dto.username).map_err(|e| AppError { message: e })?;
        is_valid_name(&dto.name).map_err(|e| AppError { message: e })?;

        // unique username
        if let Ok(existing) = self
            .find_one(None, Some(doc! { "username": &dto.username }))
            .await
        {
            return Err(AppError {
                message: format!("School username already exists: {}", existing.username),
            });
        }

        let mut new_school = dto.clone();
        new_school.is_active = Some(false);
        new_school.code = Some(generate_code());

        if let Some(logo_file) = new_school.logo.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&logo_file)
                .await
                .map_err(|e| AppError { message: e })?;
            new_school.logo_id = Some(cloud_res.public_id);
            new_school.logo = Some(cloud_res.secure_url);
        }

        let full_doc = bson::to_document(&new_school).map_err(|e| AppError {
            message: format!("Failed to serialize School: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let school = repo
            .create::<School>(extract_valid_fields(full_doc), None)
            .await?;

        let school_id = match school.id {
            Some(id) => id,
            None => {
                return Err(AppError {
                    message: "Failed to get school id".into(),
                })
            }
        };
        let db_name_clone = format!("school_{}", school_id.to_hex());

        let update_school = self
            .update(
                &IdType::from_object_id(school_id),
                &SchoolPartial {
                    database_name: Some(Some(db_name_clone)),
                    ..Default::default()
                },
            )
            .await?;

        Ok(update_school)
    }

    // =========================
    // FIND ONE (NO RELATIONS)
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<School, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<School>(filter, None)
            .await?
            .ok_or(AppError {
                message: "School not found".into(),
            })
    }

    // =========================
    // GET ALL
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<School>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "description", "code", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<School>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // =========================
    // UPDATE
    // =========================
    pub async fn update(&self, id: &IdType, update: &SchoolPartial) -> Result<School, AppError> {
        if let Some(ref name) = update.name {
            if let Err(e) = is_valid_name(name) {
                return Err(AppError { message: e });
            }
        }

        if let Some(ref username) = update.username {
            if let Err(e) = is_valid_username(username) {
                return Err(AppError { message: e });
            }
        }

        let existing = self.find_one(Some(id), None).await?;

        // name uniqueness
        if let Some(ref name) = update.name {
            if existing.name != *name {
                if let Ok(_) = self.find_one(None, Some(doc! { "name": name })).await {
                    return Err(AppError {
                        message: format!("School name already exists: {}", name),
                    });
                }
            }
        }

        // username uniqueness
        if let Some(ref username) = update.username {
            if existing.username != *username {
                if let Ok(_) = self
                    .find_one(None, Some(doc! { "username": username }))
                    .await
                {
                    return Err(AppError {
                        message: format!("School username already exists: {}", username),
                    });
                }
            }
        }

        // code uniqueness
        if let Some(code) = update.code.clone().flatten() {
            if existing.code.as_deref() != Some(&code) {
                if let Ok(school) = self.find_one(None, Some(doc! { "code": code })).await {
                    return Err(AppError {
                        message: format!("School code already exists: {:?}", school.code),
                    });
                }
            }
        }

        let mut update_data = update.clone();

        if let Some(new_logo_data) = update.logo.clone().flatten() {
            if Some(new_logo_data.clone()) != existing.logo {
                if let Some(old_logo_id) = existing.logo_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_logo_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_logo_data)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.logo_id = Some(Some(cloud_res.public_id));
                update_data.logo = Some(Some(cloud_res.secure_url));
            }
        }

        let update_doc = bson::to_document(&update_data).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<School>(id, extract_valid_fields(update_doc))
            .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<School, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let school = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(school)
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "username", "description", "code", "_id"];

        repo.count(filter, &searchable, extra_match).await
    }

    pub async fn refresh_school_token(&self, token: &str) -> Result<String, AppError> {
        // remove "Bearer " if present
        let token_clean = token.replace("Bearer ", "");
        let claims = verify_school_token(&token_clean).ok_or_else(|| AppError {
            message: "Invalid token".to_string(),
        })?;

        // get user from DB to ensure still valid
        let school_id = IdType::from_string(&claims.id);
        let school = self.find_one(Some(&school_id), None).await?;

        // create a fresh token
        let dto = to_school_school_token(&school).map_err(|e| AppError { message: e })?;
        let new_token = create_school_token(dto);

        Ok(new_token)
    }

    pub async fn create_school_token(&self, id: &IdType) -> Result<String, AppError> {
        let school = self.find_one(Some(id), None).await?;
        let school_token = to_school_school_token(&school).map_err(|e| AppError { message: e })?;
        let token = create_school_token(school_token);

        Ok(token)
    }

    pub async fn setup_school_academics(
        &self,
        school_id: &IdType,
        req: SchoolAcademicRequest,
        state: web::Data<AppState>,
        logged_user: AuthUserDto,
    ) -> Result<SchoolAcademicResponse, AppError> {
        let school_data = SchoolPartial {
            curriculum: Some(req.sector_ids.clone()),
            education_level: Some(req.trade_ids.clone()),
            ..Default::default()
        };

        let school = self.update(school_id, &school_data).await?;
        let school_db_name = school.database_name.ok_or(AppError {
            message: "School database name not found".to_string(),
        })?;

        let school_db = state.db.get_db(&school_db_name);

        let creator_id = Some(ObjectId::from_str(&logged_user.id).map_err(|e| AppError {
            message: format!("Failed to convert creator id: {}", e),
        })?);

        let class_repo_school = ClassRepo::new(&school_db);
        let class_service_school = ClassService::new(&class_repo_school);

        let class_subject_service = ClassSubjectService::new(&school_db);

        let trade_service = TradeService::new(&state.db.main_db());
        let template_subject_service = TemplateSubjectService::new(&state.db.main_db());
        let main_class_service = MainClassService::new(&state.db.main_db());

        // Academic year
        let current_year = Utc::now().year();
        let academic_year = format!("{}-{}", current_year, current_year + 1);

        let sector_ids = req.sector_ids.unwrap_or_default();
        let trade_ids = req.trade_ids.unwrap_or_default();

        let trades = match (!trade_ids.is_empty(), !sector_ids.is_empty()) {
            (true, _) => trade_service.get_trades_by_ids(&trade_ids).await?,
            (false, true) => trade_service.get_trades_by_sector_ids(&sector_ids).await?,
            _ => {
                return Err(AppError {
                    message: "No trade or sector IDs provided".to_string(),
                })
            }
        };

        let mut created_subjects_count = 0;

        let mut class_trade_pairs = Vec::new();

        for trade in trades {
            let trade_id = trade.id.ok_or_else(|| AppError {
                message: format!("Trade id not found for trade '{}'", trade.name),
            })?;

            let main_classes = main_class_service
                .get_all(
                    None,
                    None,
                    None,
                    Some(doc! {
                        "trade_id": &trade_id
                    }),
                )
                .await?
                .data;

            for main_class in main_classes {
                if main_class.disable.unwrap_or(false) {
                    continue;
                }

                let level = main_class.level.unwrap_or(1);

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
                    is_active: Some(true),
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
                    settings: Some(ClassSettings::default()),
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

        let classes_to_create: Vec<Class> = class_trade_pairs
            .iter()
            .map(|(class, _)| class.clone())
            .collect();

        let created_classes = class_service_school
            .create_many_classes(classes_to_create)
            .await
            .map_err(|e| AppError { message: e })?;

        let created_classes_count = created_classes.len();

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
            let template_subjects = template_subject_service
                .find_many_by_prerequisite(&IdType::ObjectId(main_class_id))
                .await?;

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
                let created_subjects = class_subject_service.create_many(class_subjects).await?;

                created_subjects_count += created_subjects.len();
            }
        }

        Ok(SchoolAcademicResponse {
            success: true,
            created_classes: created_classes_count,
            created_subjects: created_subjects_count,
        })
    }
}
