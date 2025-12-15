use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    api::join_school_request::create_join_school_request_controller,
    config::state::AppState,
    controller::{
        join_school_request_controller::JoinSchoolRequestController,
        teacher_controller::TeacherController,
    },
    domain::{
        auth_user::AuthUserDto,
        teacher::{
            BulkTeacherIds, BulkTeacherTags, BulkUpdateTeacherActive, PrepareTeacherRequest,
            Teacher, TeacherCountQuery, TeacherType, UpdateTeacher,
        },
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::{
        class_repo::ClassRepo, school_repo::SchoolRepo, subject_repo::SubjectRepo,
        teacher_repo::TeacherRepo, user_repo::UserRepo,
    },
    services::{
        class_service::ClassService, event_service::EventService, school_service::SchoolService,
        subject_service::SubjectService, teacher_service::TeacherService,
        user_service::UserService,
    },
    utils::api_utils::build_extra_match,
};

// Helper function to create TeacherController
fn create_teacher_controller(
    state: &web::Data<AppState>,
    claims: &SchoolToken,
) -> TeacherController<'static> {
    let school_db = state.db.get_db(&claims.database_name);
    let main_db = state.db.main_db();

    // === Repositories ===
    let teacher_repo: &'static TeacherRepo = Box::leak(Box::new(TeacherRepo::new(&school_db)));
    let school_repo: &'static SchoolRepo = Box::leak(Box::new(SchoolRepo::new(&main_db)));
    let user_repo: &'static UserRepo = Box::leak(Box::new(UserRepo::new(&main_db)));
    let subject_repo: &'static SubjectRepo = Box::leak(Box::new(SubjectRepo::new(&school_db)));
    let class_repo: &'static ClassRepo = Box::leak(Box::new(ClassRepo::new(&school_db)));

    //    === JOin School ===
    let join_school_controller: &'static JoinSchoolRequestController =
        Box::leak(Box::new(create_join_school_request_controller(state)));

    // === Services ===
    let school_service: &'static SchoolService =
        Box::leak(Box::new(SchoolService::new(school_repo)));
    let user_service: &'static UserService = Box::leak(Box::new(UserService::new(user_repo)));
    let teacher_service: &'static TeacherService =
        Box::leak(Box::new(TeacherService::new(teacher_repo)));
    let subject_service: &'static SubjectService =
        Box::leak(Box::new(SubjectService::new(subject_repo)));
    let class_service: &'static ClassService = Box::leak(Box::new(ClassService::new(class_repo)));

    // === Controller ===
    TeacherController::new(
        teacher_repo, // ✅ now a reference
        school_service,
        user_service,
        teacher_service,
        subject_service,
        class_service,
        join_school_controller,
    )
}

// Single-database operations (keep using TeacherService)
#[get("")]
async fn get_all_teachers(
    req: actix_web::HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .get_all_teachers(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_teachers(
    query: web::Query<RequestQuery>,
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .get_active_teachers(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_teacher_by_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.get_teacher_by_id(&teacher_id).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/user/{user_id}")]
async fn get_teacher_by_user_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let user_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.get_teacher_by_user_id(&user_id).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/email/{email}")]
async fn get_teacher_by_email(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let email = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.get_teacher_by_email(&email).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

// Cross-database operations (use TeacherController)
#[get("/with-relations")]
async fn get_all_teachers_with_relations(
    query: web::Query<RequestQuery>,
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_all_teachers_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/{id}/with-relations")]
async fn get_teacher_by_id_with_relations(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_teacher_by_id_with_relations(&teacher_id)
        .await
    {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/user/{user_id}/with-relations")]
async fn get_teacher_by_user_id_with_relations(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let user_id = IdType::from_string(path.into_inner());
    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_teacher_by_user_id_with_relations(&user_id)
        .await
    {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/email/{email}/with-relations")]
async fn get_teacher_by_email_with_relations(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let email = path.into_inner();
    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_teacher_by_email_with_relations(&email)
        .await
    {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/creator/{creator_id}/with-relations")]
async fn get_teachers_by_creator_id_with_relations(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let creator_id = IdType::from_string(path.into_inner());
    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_teachers_by_creator_id_with_relations(&creator_id)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

// Single-database operations (keep using TeacherService)
#[post("")]
async fn create_teacher(
    user: web::ReqData<AuthUserDto>,
    req: actix_web::HttpRequest,
    data: web::Json<Teacher>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_or_staff(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let teacher_controller = create_teacher_controller(&state, &claims);
    // Set the school_id from the token to ensure consistency
    let mut teacher_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    teacher_data.school_id = Some(school_id);

    let user_id = match ObjectId::from_str(&logged_user.id) {
        Ok(i) => i,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("field to change user id into object id: {}", e),
            })
        }
    };
    // ✅ USE THE EVENT-BASED METHOD
    match teacher_controller
        .create_teacher(teacher_data, user_id)
        .await
    {
        Ok(teacher) => {
            // clone state for each future separately
            let state_for_join_event = state.clone();
            let state_for_teacher_event = state.clone();

            // first async event
            let logged_user_id = logged_user.id.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &state_for_join_event,
                    "join_school_request",
                    "new",
                    &serde_json::json!({ "action": "created", "by_user": logged_user_id }),
                )
                .await;
            });

            // second async event
            let teacher_clone = teacher.clone();
            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_created(
                        &state_for_teacher_event,
                        "teacher",
                        &id.to_hex(),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(teacher)
        }

        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateTeacher>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD
    match service.update_teacher(&teacher_id, data.into_inner()).await {
        Ok(teacher) => {
            let state_clone = state.clone();
            let value = teacher.clone();
            actix_rt::spawn(async move {
                if let Some(id) = teacher.id {
                    EventService::broadcast_updated(&state_clone, "teacher", &id.to_hex(), &value)
                        .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .delete_teacher_with_events(&teacher_id, &state)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Teacher deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

// Single-database operations (keep using TeacherService)
#[get("/creator/{creator_id}")]
async fn get_teachers_by_creator_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let creator_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.get_teachers_by_creator_id(&creator_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/class/{class_id}")]
async fn get_teachers_by_class_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .get_teachers_by_class_id(
            &class_id,
            query.filter.clone(),
            query.limit.clone(),
            query.skip.clone(),
        )
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}")]
async fn get_teachers_by_subject_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.get_teachers_by_subject_id(&subject_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

// Cross-database operations (use TeacherController)
#[get("/type/{type}/with-relations")]
async fn get_teachers_by_type_with_relations(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let type_str = path.into_inner();
    let teacher_type = match type_str.as_str() {
        "regular" => TeacherType::Regular,
        "headteacher" => TeacherType::HeadTeacher,
        "subjectteacher" => TeacherType::SubjectTeacher,
        "deputy" => TeacherType::Deputy,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid teacher type. Must be 'regular', 'headteacher', 'subjectteacher', or 'deputy'".to_string(),
            })
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_teachers_by_type_with_relations(teacher_type)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/school/{school_id}/type/{type}/with-relations")]
async fn get_teachers_by_school_and_type_with_relations(
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let (school_id_str, type_str) = path.into_inner();
    let school_id = IdType::from_string(school_id_str);
    let teacher_type = match type_str.as_str() {
        "regular" => TeacherType::Regular,
        "headteacher" => TeacherType::HeadTeacher,
        "subjectteacher" => TeacherType::SubjectTeacher,
        "deputy" => TeacherType::Deputy,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid teacher type. Must be 'regular', 'headteacher', 'subjectteacher', or 'deputy'".to_string(),
            })
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_teachers_by_school_and_type_with_relations(&school_id, teacher_type)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

// Cross-database operations (use TeacherController)
#[get("/stats/count")]
async fn count_teachers(
    query: web::Query<TeacherCountQuery>,
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .count_teachers_by_school_id(
            &IdType::from_string(claims.id.clone()),
            query.gender.clone(),
            query.teacher_type,
        )
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/stats/count-by-type/{type}")]
async fn count_teachers_by_type(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let type_str = path.into_inner();
    let teacher_type = match type_str.as_str() {
        "regular" => TeacherType::Regular,
        "headteacher" => TeacherType::HeadTeacher,
        "subjectteacher" => TeacherType::SubjectTeacher,
        "deputy" => TeacherType::Deputy,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid teacher type. Must be 'regular', 'headteacher', 'subjectteacher', or 'deputy'".to_string(),
            })
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .count_teachers_by_type(teacher_type)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

// Single-database operations (keep using TeacherService)
#[get("/stats/count-by-creator/{creator_id}")]
async fn count_teachers_by_creator_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let creator_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.count_teachers_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

// Single-database operations (keep using TeacherService)
#[get("/stats/count-by-class/{class_id}")]
async fn count_teachers_by_class_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.count_teachers_by_class_id(&class_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-subject/{subject_id}")]
async fn count_teachers_by_subject_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let subject_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.count_teachers_by_subject_id(&subject_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

// Bulk operations using TeacherController
#[post("/bulk")]
async fn create_many_teachers(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<Teacher>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    // Set school_id for all teachers
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    let mut teachers_with_school = data.into_inner();
    for teacher in &mut teachers_with_school {
        teacher.school_id = Some(school_id);
    }

    match teacher_controller
        .create_many_teachers(teachers_with_school)
        .await
    {
        Ok(teachers) => {
            let state_clone = state.clone();
            let teachers_for_spawn = teachers.clone();

            actix_rt::spawn(async move {
                for teacher in &teachers_for_spawn {
                    if let Some(id) = teacher.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "teacher",
                            &id.to_hex(),
                            teacher,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(teachers)
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

// Bulk operations using TeacherController
#[put("/bulk/active")]
async fn bulk_update_teacher_active(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateTeacherActive>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .bulk_update_active(&data.into_inner())
        .await
    {
        Ok(teachers) => {
            let state_clone = state.clone();
            let teachers_for_spawn = teachers.clone();

            actix_rt::spawn(async move {
                for teacher in &teachers_for_spawn {
                    if let Some(id) = teacher.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "teacher",
                            &id.to_hex(),
                            teacher,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(teachers)
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[put("/bulk/add-tags")]
async fn bulk_add_tags_to_teachers(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTeacherTags>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller.bulk_add_tags(&data.into_inner()).await {
        Ok(teachers) => {
            let state_clone = state.clone();
            let teachers_for_spawn = teachers.clone();

            actix_rt::spawn(async move {
                for teacher in &teachers_for_spawn {
                    if let Some(id) = teacher.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "teacher",
                            &id.to_hex(),
                            teacher,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(teachers)
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[put("/bulk/remove-tags")]
async fn bulk_remove_tags_from_teachers(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTeacherTags>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .bulk_remove_tags(&data.into_inner())
        .await
    {
        Ok(teachers) => {
            let state_clone = state.clone();
            let teachers_for_spawn = teachers.clone();

            actix_rt::spawn(async move {
                for teacher in &teachers_for_spawn {
                    if let Some(id) = teacher.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "teacher",
                            &id.to_hex(),
                            teacher,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(teachers)
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[delete("/bulk")]
async fn delete_many_teachers(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTeacherIds>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.delete_many_teachers(&data.into_inner()).await {
        Ok(deleted_count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully deleted {} teachers", deleted_count)
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

// Cross-database operations (use TeacherController)
#[get("/check/{user_id}")]
async fn is_user_teacher_of_school(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let user_id = IdType::from_string(path.into_inner());
    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .is_user_teacher_of_school(&user_id, &IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(is_teacher) => HttpResponse::Ok().json(serde_json::json!({ "is_teacher": is_teacher })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

// Cross-database operations (use TeacherController)
#[get("/stats/school-statistics")]
async fn get_school_teacher_statistics(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_school_teacher_statistics(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

// Cross-database operations (use TeacherController)
#[get("/head-teachers/with-relations")]
async fn get_school_head_teachers_with_relations(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_school_head_teachers_with_relations(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/subject-teachers/with-relations")]
async fn get_school_subject_teachers_with_relations(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_school_subject_teachers_with_relations(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/deputy-teachers/with-relations")]
async fn get_school_deputy_teachers_with_relations(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_school_deputy_teachers_with_relations(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/regular-teachers/with-relations")]
async fn get_school_regular_teachers_with_relations(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    match teacher_controller
        .get_school_regular_teachers_with_relations(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

// Single-database operations (keep using TeacherService)
#[put("/{id}/activate")]
async fn activate_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.activate_teacher(&teacher_id).await {
        Ok(teacher) => {
            // Broadcast teacher update event
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/deactivate")]
async fn deactivate_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.deactivate_teacher(&teacher_id).await {
        Ok(teacher) => {
            // Broadcast teacher update event
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Remove classes from teacher
#[put("/{id}/remove-classes")]
async fn remove_classes_from_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    // Convert string IDs to ObjectIds
    let class_ids: Result<Vec<ObjectId>, _> = data
        .into_inner()
        .into_iter()
        .map(|id| ObjectId::from_str(&id))
        .collect();

    let class_ids = match class_ids {
        Ok(ids) => ids,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Invalid class ID format: {}", e),
            });
        }
    };

    match service
        .remove_classes_from_teacher(&teacher_id, class_ids)
        .await
    {
        Ok(teacher) => {
            // Broadcast teacher update event
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Remove subjects from teacher
#[put("/{id}/remove-subjects")]
async fn remove_subjects_from_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    // Convert string IDs to ObjectIds
    let subject_ids: Result<Vec<ObjectId>, _> = data
        .into_inner()
        .into_iter()
        .map(|id| ObjectId::from_str(&id))
        .collect();

    let subject_ids = match subject_ids {
        Ok(ids) => ids,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Invalid subject ID format: {}", e),
            });
        }
    };

    match service
        .remove_subjects_from_teacher(&teacher_id, subject_ids)
        .await
    {
        Ok(teacher) => {
            // Broadcast teacher update event
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare teachers for bulk creation
#[post("/bulk/prepare")]
async fn prepare_teachers_for_bulk_creation(
    req: actix_web::HttpRequest,
    data: web::Json<PrepareTeacherRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.prepare_teachers(&data.into_inner()).await {
        Ok(prepared_teachers) => HttpResponse::Ok().json(prepared_teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

// Bulk operations using TeacherController
#[put("/bulk-update")]
async fn update_many_teachers(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<(String, UpdateTeacher)>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let teacher_controller = create_teacher_controller(&state, &claims);

    // Convert the request data to the format expected by the service
    let updates: Vec<(IdType, UpdateTeacher)> = data
        .into_inner()
        .into_iter()
        .map(|(id_str, update)| (IdType::from_string(id_str), update))
        .collect();

    match teacher_controller.update_many_teachers(updates).await {
        Ok(updated_teachers) => {
            let state_clone = state.clone();
            let teachers_for_spawn = updated_teachers.clone();

            actix_rt::spawn(async move {
                for teacher in &teachers_for_spawn {
                    if let Some(id) = teacher.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "teacher",
                            &id.to_hex(),
                            teacher,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(updated_teachers)
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/count")]
async fn count_teachers_fields(
    req: actix_web::HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    // ===== SCHOOL DB =====
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_teachers(query.filter.clone(), extra_match)
        .await
    {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!(total)),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

// Single-database operations (keep using TeacherService)
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/teachers")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // =============================================
            // PUBLIC ROUTES (READ-ONLY)
            // =============================================
            // Teacher Listing & Retrieval (Single Database)
            .service(get_all_teachers) // GET    /school/teachers - Get all teachers with optional filtering
            .service(get_active_teachers) // GET    /school/teachers/active - Get only active teachers
            .service(get_teacher_by_user_id) // GET    /school/teachers/user/{user_id} - Get teacher by user ID
            .service(get_teacher_by_email) // GET    /school/teachers/email/{email} - Get teacher by email
            .service(get_teachers_by_creator_id) // GET    /school/teachers/creator/{creator_id} - Get teachers by creator ID
            .service(get_teachers_by_class_id) // GET    /school/teachers/class/{class_id} - Get teachers by class ID
            .service(get_teachers_by_subject_id) // GET    /school/teachers/subject/{subject_id} - Get teachers by subject ID
            // Teacher Listing & Retrieval (Cross Database - with relations)
            .service(count_teachers_fields) // GET    /school/teachers/count - Count teachers with optional filters
            .service(get_all_teachers_with_relations) // GET    /school/teachers/with-relations - Get all teachers with user/school relations
            .service(get_teacher_by_id_with_relations) // GET    /school/teachers/{id}/with-relations - Get teacher by ID with full relations
            .service(get_teacher_by_user_id_with_relations) // GET    /school/teachers/user/{user_id}/with-relations - Get teacher by user ID with relations
            .service(get_teacher_by_email_with_relations) // GET    /school/teachers/email/{email}/with-relations - Get teacher by email with relations
            .service(get_teachers_by_creator_id_with_relations) // GET    /school/teachers/creator/{creator_id}/with-relations - Get teachers by creator ID with relations
            .service(get_teachers_by_type_with_relations) // GET    /school/teachers/type/{type}/with-relations - Get teachers by type with relations
            .service(get_teachers_by_school_and_type_with_relations) // GET    /school/teachers/school/{school_id}/type/{type}/with-relations - Get teachers by school and type with relations
            // Specialized Teacher Types (Cross Database - with relations)
            .service(get_school_head_teachers_with_relations) // GET    /school/teachers/head-teachers/with-relations - Get head teachers with relations
            .service(get_school_subject_teachers_with_relations) // GET    /school/teachers/subject-teachers/with-relations - Get subject teachers with relations
            .service(get_school_deputy_teachers_with_relations) // GET    /school/teachers/deputy-teachers/with-relations - Get deputy teachers with relations
            .service(get_school_regular_teachers_with_relations) // GET    /school/teachers/regular-teachers/with-relations - Get regular teachers with relations
            // Permission Checking (Cross Database)
            .service(is_user_teacher_of_school) // GET    /school/teachers/check/{user_id} - Check if user is teacher of school
            // Statistics & Analytics (Mixed)
            .service(count_teachers) // GET    /school/teachers/stats/count - Count teachers with optional filters
            .service(count_teachers_by_creator_id) // GET    /school/teachers/stats/count-by-creator/{creator_id} - Count teachers by creator ID
            .service(count_teachers_by_type) // GET    /school/teachers/stats/count-by-type/{type} - Count teachers by type
            .service(count_teachers_by_class_id) // GET    /school/teachers/stats/count-by-class/{class_id} - Count teachers by class ID
            .service(count_teachers_by_subject_id) // GET    /school/teachers/stats/count-by-subject/{subject_id} - Count teachers by subject ID
            .service(get_school_teacher_statistics) // GET    /school/teachers/stats/school-statistics - Get comprehensive teacher statistics
            .service(get_teacher_by_id) // GET    /school/teachers/{id} - Get teacher by ID
            // =============================================
            // PROTECTED ROUTES (WRITE OPERATIONS)
            // =============================================
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            // Single Teacher Operations (Single Database)
            .service(create_teacher) // POST   /school/teachers - Create new teacher
            .service(update_teacher) // PUT    /school/teachers/{id} - Update teacher by ID
            .service(delete_teacher) // DELETE /school/teachers/{id} - Delete teacher by ID
            // Teacher Status Management (Single Database)
            .service(activate_teacher) // PUT    /school/teachers/{id}/activate - Activate teacher
            .service(deactivate_teacher) // PUT    /school/teachers/{id}/deactivate - Deactivate teacher
            // =============================================
            // BULK OPERATIONS
            // =============================================
            // Bulk Creation (Mixed)
            .service(create_many_teachers) // POST   /school/teachers/bulk - Create multiple teachers (cross-database)
            // Bulk Updates (Cross Database)
            .service(update_many_teachers) // PUT    /school/teachers/bulk-update - Update multiple teachers
            .service(bulk_update_teacher_active) // PUT    /school/teachers/bulk/active - Bulk update teacher active status
            .service(bulk_add_tags_to_teachers) // PUT    /school/teachers/bulk/add-tags - Bulk add tags to teachers
            .service(bulk_remove_tags_from_teachers) // PUT    /school/teachers/bulk/remove-tags - Bulk remove tags from teachers
            // Bulk Deletion (Single Database)
            .service(delete_many_teachers), // DELETE /school/teachers/bulk - Delete multiple teachers
    );
}
