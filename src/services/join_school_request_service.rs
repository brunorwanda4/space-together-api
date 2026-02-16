use std::str::FromStr;

use actix_web::web;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use chrono::{DateTime, Datelike, Utc};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        common_details::Paginated,
        join_school_request::{
            CreateJoinSchoolRequest, JoinRole, JoinSchoolByCode, JoinSchoolRequest,
            JoinSchoolRequestResponseToken, JoinSchoolRequestWithRelations, JoinStatus,
        },
        school_staff::{parse_staff_type, SchoolStaff, SchoolStaffType},
        student::{Student, StudentPartial, StudentStatus},
        teacher::{parse_teacher_type, Teacher},
        user::User,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::join_school_request_pipeline::join_school_request_pipeline,
    repositories::{base_repo::BaseRepository, user_repo::UserRepo},
    services::{
        class_service::ClassService, school_service::SchoolService,
        school_staff_service::SchoolStaffService, student_service::StudentService,
        teacher_service::TeacherService, user_service::UserService,
    },
    utils::{
        code::generate_school_registration_number, email::is_valid_email,
        mongo_utils::build_search_filter,
    },
};

pub struct JoinSchoolRequestService {
    pub collection: Collection<JoinSchoolRequest>,
}

impl JoinSchoolRequestService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<JoinSchoolRequest>("join_school_requests"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("email", false),
            IndexDef::single("school_id", false),
            IndexDef::single("class_id", false),
            IndexDef::single("status", false),
            IndexDef::compound(vec![("email", 1), ("school_id", 1), ("status", 1)], false),
            IndexDef::single("expires_at", false),
            IndexDef::single("created_at", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(
        &self,
        request: CreateJoinSchoolRequest,
        sent_by: ObjectId,
        state: &AppState,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.ensure_indexes().await?;
        if let Err(e) = is_valid_email(&request.email) {
            return Err(AppError { message: e });
        }

        if let JoinRole::Student = request.role.clone() {
            if request.class_id.is_none() {
                return Err(AppError {
                    message: "Class is required for student join requests".into(),
                });
            }
        }

        let school_id = ObjectId::from_str(&request.school_id.clone()).map_err(|e| AppError {
            message: format!("Failed to change school id into objectId:{}", e),
        })?;

        if (self.find_one(None, Some(doc! {"email": request.email.clone(), "school_id": school_id.clone(), "status": bson::to_bson(&JoinStatus::Pending).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?})))
            .await
            .is_ok()
        {
            return Err(AppError {
                message: "A join request with this email already exists".into(),
            });
        }

        let user_repo = UserRepo::new(&state.db.main_db());
        let user_service = UserService::new(&user_repo);

        let mut invited_user_id = None;
        if let Ok(user) = user_service
            .get_user_by_email(&request.email)
            .await
            .map_err(|e| AppError { message: e })
        {
            if let Some(schools) = &user.schools {
                if schools.contains(&school_id.clone()) {
                    return Err(AppError {
                        message: "User is already a member of this school".into(),
                    });
                }
            }

            invited_user_id = user.id;
        }

        let class_id = match request.class_id {
            Some(class_id_str) => {
                Some(ObjectId::from_str(&class_id_str).map_err(|e| AppError {
                    message: format!("Failed to change class id into objectId:{}", e),
                })?)
            }
            None => None,
        };

        let now = Utc::now();
        let join_request = JoinSchoolRequest {
            id: None,
            school_id,
            invited_user_id,
            class_id,
            role: request.role.clone(),
            email: request.email.clone(),
            r#type: request.r#type.clone(),
            message: request.message,
            status: JoinStatus::Pending,
            sent_at: now,
            responded_at: None,
            expires_at: Some(now + chrono::Duration::days(7)),
            sent_by,
            created_at: now,
            updated_at: now,
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let doc = mongodb::bson::to_document(&join_request).map_err(|e| AppError {
            message: format!("Serialize join request failed: {}", e),
        })?;

        repo.create::<JoinSchoolRequest>(doc, None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<JoinSchoolRequest>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Join school request not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<JoinSchoolRequest>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "email",
            "type",
            "role",
            "status",
            "message",
            "_id",
            "school_id",
            "class_id",
            "invited_user_id",
            "sent_by",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<JoinSchoolRequest>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn accept_request(
        &self,
        id: &IdType,
        invited_user_id: ObjectId,
        responded_by: Option<ObjectId>,
        state: web::Data<AppState>,
        logged_user: &AuthUserDto,
    ) -> Result<JoinSchoolRequestResponseToken, AppError> {
        let request = self.find_one(Some(id), None).await?;
        if !matches!(request.status, JoinStatus::Pending) {
            return Err(AppError {
                message: "Join request is not pending".into(),
            });
        }

        let user_repo = UserRepo::new(&state.db.main_db());
        let user_service = UserService::new(&user_repo);

        let user = user_service
            .get_user_by_email(&request.email)
            .await
            .map_err(|e| AppError { message: e })?;

        let _ = self
            .create_role_entity_school_db(&request, &user, &state)
            .await;

        self.update_status(
            id,
            JoinStatus::Accepted,
            Some(invited_user_id),
            responded_by,
        )
        .await?;

        let school_service = SchoolService::new(&state.db.main_db());

        let school_token = school_service
            .create_school_token(&IdType::ObjectId(request.school_id), logged_user, &state)
            .await?;

        Ok(JoinSchoolRequestResponseToken { school_token })
    }

    pub async fn reject_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Rejected, None, responded_by)
            .await
    }

    pub async fn cancel_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Cancelled, None, responded_by)
            .await
    }

    async fn update_status(
        &self,
        id: &IdType,
        status: JoinStatus,
        invited_user_id: Option<ObjectId>,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let mut update = doc! {
            "status": bson::to_bson(&status).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?,
        };

        if let Some(uid) = invited_user_id {
            update.insert("invited_user_id", uid);
        }

        if let Some(by) = responded_by {
            update.insert("responded_by", by);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<JoinSchoolRequest>(id, update)
            .await
    }

    pub async fn update_expiration(
        &self,
        id: &IdType,
        expires_at: DateTime<Utc>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<JoinSchoolRequest>(
            id,
            doc! {
                "expires_at": bson::to_bson(&expires_at).unwrap(),
            },
        )
        .await
    }

    pub async fn expire_old_requests(&self) -> Result<u64, AppError> {
        let now = Utc::now();
        let result = self
            .collection
            .update_many(
                doc! {
                    "expires_at": { "$lte": bson::to_bson(&now).unwrap() },
                    "status": bson::to_bson(&JoinStatus::Pending).map_err(|e| AppError {
                        message: format!("Failed to serialize status: {}", e),
                    })?
                },
                doc! {
                    "$set": {
                        "status": bson::to_bson(&JoinStatus::Expired).map_err(|e| AppError {
                            message: format!("Failed to serialize status: {}", e),
                        })?,
                        "updated_at": bson::to_bson(&now).unwrap()
                    }
                },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to expire old join requests: {}", e),
            })?;

        Ok(result.modified_count)
    }

    pub async fn cleanup_expired_requests(
        &self,
        older_than_days: i64,
    ) -> Result<CountDoc, AppError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days);
        let result = self
            .collection
            .delete_many(doc! {
                "status": bson::to_bson(&JoinStatus::Expired).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?,
                "updated_at": { "$lte": bson::to_bson(&cutoff_date).unwrap() }
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to cleanup expired join requests: {}", e),
            })?;

        Ok(CountDoc {
            count: result.deleted_count,
        })
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<JoinSchoolRequestWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &[
                    "email",
                    "type",
                    "role",
                    "status",
                    "message",
                    "_id",
                    "school_id",
                    "class_id",
                    "invited_user_id",
                    "sent_by",
                ],
            );

            match_stage.extend(search);
        }

        let pipeline = join_school_request_pipeline(match_stage);

        repo.aggregate_with_paginate::<JoinSchoolRequestWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<JoinSchoolRequestWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<JoinSchoolRequestWithRelations>(
            join_school_request_pipeline(match_stage),
            None,
        )
        .await?
        .ok_or(AppError {
            message: "Join school request not found".into(),
        })
    }

    async fn create_role_entity_school_db(
        &self,
        request: &JoinSchoolRequest,
        user: &User,
        state: &web::Data<AppState>,
    ) -> Result<(), AppError> {
        let user_id = user.id.unwrap();
        let school_id = request.school_id;
        let school_service = SchoolService::new(&state.db.main_db());
        let school = school_service
            .find_one(Some(&IdType::ObjectId(school_id)), None)
            .await?;

        let school_db_name = school.database_name.as_ref().ok_or_else(|| AppError {
            message: "School database not configured".into(),
        })?;

        let school_db = state.db.get_db(school_db_name);

        match request.role {
            JoinRole::Student => {
                let student_service = StudentService::new(&school_db);
                let class_service = ClassService::new(&school_db);

                if let Some(class_id) = request.class_id {
                    if class_service
                        .find_one(Some(&IdType::ObjectId(class_id)), None)
                        .await
                        .is_err()
                    {
                        return Err(AppError {
                            message: "Class not found in school database".into(),
                        });
                    }
                }

                if let Ok(student) = student_service
                    .find_one(None, Some(doc! {"email": user.email.clone()}))
                    .await
                {
                    let update_student = StudentPartial {
                        id: None,               // StudentPartial.id is Option<ObjectId>
                        user_id: Some(user.id), // Assuming user.id is ObjectId
                        school_id: Some(school.id),
                        class_id: Some(request.class_id),
                        subclass_id: Some(student.subclass_id), // Already Option<ObjectId>
                        creator_id: Some(student.creator_id),   // Already Option<ObjectId>

                        name: Some(student.name), // String -> Option<String>
                        email: Some(student.email), // String -> Option<String>

                        phone: Some(student.phone.or(user.phone.clone())),
                        gender: Some(student.gender.or(user.gender.clone())),
                        image: Some(student.image.or(user.image.clone())),
                        image_id: Some(student.image_id.or(user.image_id.clone())),

                        date_of_birth: Some(student.date_of_birth.or(user.age.clone())),

                        registration_number: Some(
                            student
                                .registration_number
                                .or(generate_school_registration_number(&school)),
                        ), // Already Option
                        admission_year: Some(student.admission_year), // Already Option

                        status: Some(student.status), // StudentStatus -> Option<StudentStatus>
                        is_active: Some(student.is_active), // bool -> Option<bool>
                        tags: Some(student.tags),     // Vec -> Option<Vec>

                        created_at: None,
                        updated_at: None,
                        deleted_at: None,
                        deleted_by: None,
                    };

                    student_service
                        .update(
                            &IdType::from_object_id(student.id.unwrap()),
                            &update_student,
                        )
                        .await?;
                } else {
                    let new_student = Student {
                        id: None,
                        user_id: Some(user_id),
                        school_id: Some(school_id),
                        class_id: request.class_id,
                        creator_id: Some(request.sent_by),
                        name: user.name.clone(),
                        email: user.email.clone(),
                        phone: user.phone.clone(),
                        gender: user.gender.clone(),
                        date_of_birth: user.age.clone(),
                        subclass_id: None,
                        image: user.image.clone(),
                        image_id: user.image_id.clone(),
                        registration_number: generate_school_registration_number(&school),
                        admission_year: Some(Utc::now().year()),
                        status: StudentStatus::Active,
                        is_active: false,
                        tags: vec!["join-request".to_string()],
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        deleted_at: None,
                        deleted_by: None,
                    };
                    student_service.create(new_student, None).await?;
                }
            }

            JoinRole::Teacher => {
                let teacher_service = TeacherService::new(&school_db);
                let teacher_type = parse_teacher_type(&request.r#type);

                let new_teacher = Teacher {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    phone: user.phone.clone(),
                    gender: user.gender.clone(),
                    r#type: teacher_type,
                    class_ids: None,
                    subject_ids: None,
                    is_active: false,
                    image: user.image.clone(),
                    image_id: user.image_id.clone(),
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                teacher_service.create(new_teacher, None).await?;
            }

            JoinRole::Staff => {
                let staff_service = SchoolStaffService::new(&school_db);

                let staff_type = parse_staff_type(&request.r#type);

                match staff_type {
                    SchoolStaffType::Director => {
                        let count = staff_service
                            .count_staff(None, Some(doc! {"type": "Director"}))
                            .await?;

                        if count.count >= 1 {
                            return Err(AppError {
                                message: "This school already has a Director".into(),
                            });
                        }
                    }
                    SchoolStaffType::HeadOfStudies => {
                        let count = staff_service
                            .count_staff(None, Some(doc! {"type": "HeadOfStudies"}))
                            .await?;

                        if count.count >= 5 {
                            return Err(AppError {
                                message: "This school already has 5 HeadOfStudies".into(),
                            });
                        }
                    }
                }

                let staff = SchoolStaff {
                    id: None,
                    user_id: Some(user_id),
                    school_id: Some(school_id),
                    creator_id: Some(request.sent_by),
                    name: user.name.clone(),
                    email: user.email.clone(),
                    image: user.image.clone(),
                    image_id: user.image_id.clone(),
                    r#type: staff_type,
                    is_active: false,
                    tags: vec!["join-request".to_string()],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                staff_service.create(staff, None).await?;
            }
        }

        Ok(())
    }

    pub async fn join_school_by_code(
        &self,
        request: &JoinSchoolByCode,
        auth_user: &AuthUserDto,
        state: web::Data<AppState>,
    ) -> Result<JoinSchoolRequestResponseToken, AppError> {
        let user_repo = UserRepo::new(&state.db.main_db());
        let user_service = UserService::new(&user_repo);
        let user = user_service
            .get_user_by_email(&auth_user.email)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find user: {}", e),
            })?;

        let school_service = SchoolService::new(&state.db.main_db());
        let school = school_service
            .find_one(None, Some(doc! {"code": &request.code}))
            .await?;

        let school_id = school.id.clone().ok_or_else(|| AppError {
            message: "School ID not found".into(),
        })?;

        let user_id = user.id.clone().ok_or_else(|| AppError {
            message: "User ID not found".into(),
        })?;

        let join_request = JoinSchoolRequest::new(&user, &school_id, &user_id);

        self.create_role_entity_school_db(&join_request, &user, &state)
            .await?;

        user_service
            .add_school_to_user(
                &IdType::ObjectId(user_id),
                &IdType::from_object_id(school_id.clone()),
            )
            .await
            .map_err(|e| AppError { message: e })?;

        let school_service = SchoolService::new(&state.db.main_db());

        let school_token = school_service
            .create_school_token(&IdType::ObjectId(school_id), auth_user, &state)
            .await?;

        Ok(JoinSchoolRequestResponseToken { school_token })
    }

    pub async fn count(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "email",
            "type",
            "role",
            "status",
            "message",
            "_id",
            "school_id",
            "class_id",
            "invited_user_id",
            "sent_by",
        ];

        repo.count(filter, &searchable, extra_match).await
    }

    pub async fn get_my_pending_request(
        &self,
        user_email: &str,
    ) -> Result<Paginated<JoinSchoolRequestWithRelations>, AppError> {
        let filter = doc! {
            "email": user_email,
            "status": bson::to_bson(&JoinStatus::Pending).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?
        };

        self.get_all_with_relations(None, None, None, Some(filter))
            .await
    }
}
