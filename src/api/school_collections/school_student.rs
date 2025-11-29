use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    api::join_school_request::create_join_school_request_controller,
    config::state::AppState,
    controller::{
        join_school_request_controller::JoinSchoolRequestController,
        student_controller::StudentController,
    },
    domain::{
        auth_user::AuthUserDto,
        student::{
            BulkStudentIds, BulkStudentTags, BulkUpdateStudentStatus, Student, StudentCountQuery,
            StudentStatus, UpdateStudent,
        },
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::{
        class_repo::ClassRepo, school_repo::SchoolRepo, student_repo::StudentRepo,
        user_repo::UserRepo,
    },
    services::{
        class_service::ClassService, event_service::EventService, school_service::SchoolService,
        student_service::StudentService, user_service::UserService,
    },
};

fn create_student_controller(
    state: &web::Data<AppState>,
    claims: &SchoolToken,
) -> StudentController<'static> {
    // === Databases ===
    let school_db = state.db.get_db(&claims.database_name);
    let main_db = state.db.main_db();

    // === Repositories ===
    let student_repo: &'static StudentRepo = Box::leak(Box::new(StudentRepo::new(&school_db)));
    let school_repo: &'static SchoolRepo = Box::leak(Box::new(SchoolRepo::new(&main_db)));
    let user_repo: &'static UserRepo = Box::leak(Box::new(UserRepo::new(&main_db)));
    let class_repo: &'static ClassRepo = Box::leak(Box::new(ClassRepo::new(&school_db)));

    // === Join School ===
    let join_school_controller: &'static JoinSchoolRequestController =
        Box::leak(Box::new(create_join_school_request_controller(state)));

    // === Services ===
    let school_service: &'static SchoolService =
        Box::leak(Box::new(SchoolService::new(school_repo)));
    let user_service: &'static UserService = Box::leak(Box::new(UserService::new(user_repo)));
    let student_service: &'static StudentService =
        Box::leak(Box::new(StudentService::new(student_repo)));
    let class_service: &'static ClassService = Box::leak(Box::new(ClassService::new(class_repo)));

    // === Controller ===
    StudentController::new(
        student_repo,
        student_service,
        user_service,
        school_service,
        class_service,
        join_school_controller,
    )
}

#[get("")]
async fn get_all_students(
    req: actix_web::HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Get school claims from extensions (set by SchoolTokenMiddleware)
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    // Use school database from claims
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_all_students(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_students(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_active_students().await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_student_by_id(
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

    let student_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_student_by_id(&student_id).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/user/{user_id}")]
async fn get_student_by_user_id(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_student_by_user_id(&user_id).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/email/{email}")]
async fn get_student_by_email(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_student_by_email(&email).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/registration/{registration_number}")]
async fn get_student_by_registration_number(
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

    let registration_number = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_student_by_registration_number(&registration_number)
        .await
    {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_student(
    user: web::ReqData<AuthUserDto>,
    req: actix_web::HttpRequest,
    data: web::Json<Student>,
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

    let student_controller = create_student_controller(&state, &claims);
    // Set the school_id from the token to ensure consistency
    let mut student_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    student_data.school_id = Some(school_id);

    let user_id = match ObjectId::from_str(&logged_user.id) {
        Ok(i) => i,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("field to change user id into object id: {}", e),
            })
        }
    };

    // ✅ USE THE EVENT-BASED METHOD
    match student_controller
        .create_student(student_data, user_id)
        .await
    {
        Ok(student) => {
            // clone state for each future separately
            let state_for_join_event = state.clone();
            let state_for_student_event = state.clone();

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
            let student_clone = student.clone();
            actix_rt::spawn(async move {
                if let Some(id) = student_clone.id {
                    EventService::broadcast_created(
                        &state_for_student_event,
                        "student",
                        &id.to_hex(),
                        &student_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(student)
        }

        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_student(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateStudent>,
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

    let student_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD
    match service.update_student(&student_id, data.into_inner()).await {
        Ok(student) => {
            let student_clone = student.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = student_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "student",
                        &id.to_hex(),
                        &student_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(student)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_student(
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

    let student_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .delete_student_with_events(&student_id, &state)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Student deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count")]
async fn count_students(
    req: actix_web::HttpRequest,
    query: web::Query<StudentCountQuery>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .count_students_by_school_id(
            &IdType::from_string(claims.id.clone()),
            query.gender.clone(),
            query.status,
        )
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-details")]
async fn get_all_students_with_details(
    req: actix_web::HttpRequest,
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

    let student_controller = create_student_controller(&state, &claims);

    match student_controller
        .get_all_students_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(error) => HttpResponse::BadRequest().json(ReqErrModel {
            message: error.message,
        }),
    }
}

#[get("/{id}/with-details")]
async fn get_student_by_id_with_details(
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

    let student_id = IdType::from_string(path.into_inner());
    let student_controller = create_student_controller(&state, &claims);

    match student_controller
        .get_student_by_id_with_relations(&student_id)
        .await
    {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(err) => HttpResponse::NotFound().json(ReqErrModel {
            message: err.message,
        }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_students_by_creator_id(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_students_by_creator_id(&creator_id).await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/class/{class_id}")]
async fn get_students_by_class_id(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_students_by_class_id(&class_id, query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/status/{status}")]
async fn get_students_by_status(
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

    let status_str = path.into_inner();
    let status = match status_str.as_str() {
        "active" => StudentStatus::Active,
        "suspended" => StudentStatus::Suspended,
        "graduated" => StudentStatus::Graduated,
        "left" => StudentStatus::Left,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid status. Must be 'active', 'suspended', 'graduated', or 'left'"
                    .to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_students_by_status(status).await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_students_by_creator_id(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.count_students_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-status/{status}")]
async fn count_students_by_status(
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

    let status_str = path.into_inner();
    let status = match status_str.as_str() {
        "active" => StudentStatus::Active,
        "suspended" => StudentStatus::Suspended,
        "graduated" => StudentStatus::Graduated,
        "left" => StudentStatus::Left,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid status. Must be 'active', 'suspended', 'graduated', or 'left'"
                    .to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.count_students_by_status(status).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-class/{class_id}")]
async fn count_students_by_class_id(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.count_students_by_class_id(&class_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple students for school
#[post("/bulk")]
async fn create_many_students(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<Student>>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    // Set school_id for all students
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    let mut students_with_school = data.into_inner();
    for student in &mut students_with_school {
        student.school_id = Some(school_id);
    }

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .create_many_students_with_events(students_with_school, &state)
        .await
    {
        Ok(students) => HttpResponse::Created().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update status for multiple students
#[put("/bulk/status")]
async fn bulk_update_student_status(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateStudentStatus>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.bulk_update_status(&data.into_inner()).await {
        Ok(students) => {
            let state_clone = state.clone();
            let students_for_spawn = students.clone();

            actix_rt::spawn(async move {
                for student in &students_for_spawn {
                    if let Some(id) = student.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            student,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(students)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk add tags to multiple students
#[put("/bulk/add-tags")]
async fn bulk_add_tags_to_students(
    req: actix_web::HttpRequest,
    data: web::Json<BulkStudentTags>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.bulk_add_tags(&data.into_inner()).await {
        Ok(students) => {
            let state_clone = state.clone();
            let students_for_spawn = students.clone();

            actix_rt::spawn(async move {
                for student in &students_for_spawn {
                    if let Some(id) = student.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            student,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(students)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk remove tags from multiple students
#[put("/bulk/remove-tags")]
async fn bulk_remove_tags_from_students(
    req: actix_web::HttpRequest,
    data: web::Json<BulkStudentTags>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.bulk_remove_tags(&data.into_inner()).await {
        Ok(students) => {
            let state_clone = state.clone();
            let students_for_spawn = students.clone();

            actix_rt::spawn(async move {
                for student in &students_for_spawn {
                    if let Some(id) = student.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            student,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(students)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Delete multiple students
#[delete("/bulk")]
async fn delete_many_students(
    req: actix_web::HttpRequest,
    data: web::Json<BulkStudentIds>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.delete_many_students(&data.into_inner()).await {
        Ok(deleted_count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully deleted {} students", deleted_count)
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Transfer students to another class
#[put("/bulk/transfer-class")]
async fn transfer_students_to_class(
    req: actix_web::HttpRequest,
    data: web::Json<(BulkStudentIds, String)>, // (student_ids, new_class_id)
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

    let (student_ids, new_class_id_str) = data.into_inner();
    let new_class_id = IdType::from_string(new_class_id_str);

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .transfer_students_to_class(&student_ids, &new_class_id)
        .await
    {
        Ok(students) => {
            let state_clone = state.clone();
            let students_for_spawn = students.clone();

            actix_rt::spawn(async move {
                for student in &students_for_spawn {
                    if let Some(id) = student.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            student,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(students)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if user is student of this school
#[get("/check/{user_id}")]
async fn is_user_student_of_school(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .is_user_student_of_school(&user_id, &IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(is_student) => HttpResponse::Ok().json(serde_json::json!({ "is_student": is_student })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if student is in a specific class
#[get("/check/{student_id}/class/{class_id}")]
async fn is_student_in_class(
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

    let (student_id_str, class_id_str) = path.into_inner();
    let student_id = IdType::from_string(student_id_str);
    let class_id = IdType::from_string(class_id_str);

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.is_student_in_class(&student_id, &class_id).await {
        Ok(in_class) => HttpResponse::Ok().json(serde_json::json!({ "in_class": in_class })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get class roster with student details
#[get("/class/{class_id}/roster")]
async fn get_class_roster(
    query: web::Query<RequestQuery>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_class_roster(&class_id, query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(roster) => HttpResponse::Ok().json(roster),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get student statistics for a school
#[get("/stats/school-statistics")]
async fn get_school_student_statistics(
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_school_student_statistics(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Suspend a student
#[put("/{id}/suspend")]
async fn suspend_student(
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

    let student_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.suspend_student(&student_id).await {
        Ok(student) => {
            // Broadcast student update event
            let student_clone = student.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = student_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "student",
                        &id.to_hex(),
                        &student_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(student)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Activate a student
#[put("/{id}/activate")]
async fn activate_student(
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

    let student_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.activate_student(&student_id).await {
        Ok(student) => {
            // Broadcast student update event
            let student_clone = student.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = student_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "student",
                        &id.to_hex(),
                        &student_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(student)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Graduate a student
#[put("/{id}/graduate")]
async fn graduate_student(
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

    let student_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.graduate_student(&student_id).await {
        Ok(student) => {
            // Broadcast student update event
            let student_clone = student.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = student_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "student",
                        &id.to_hex(),
                        &student_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(student)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update multiple students
#[put("/bulk-update")]
async fn update_many_students(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<(String, UpdateStudent)>>,
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
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    // Convert the request data to the format expected by the service
    let updates: Vec<(IdType, UpdateStudent)> = data
        .into_inner()
        .into_iter()
        .map(|(id_str, update)| (IdType::from_string(id_str), update))
        .collect();

    match service.update_many_students(updates).await {
        Ok(updated_students) => {
            let state_clone = state.clone();
            let students_for_spawn = updated_students.clone();

            actix_rt::spawn(async move {
                for student in &students_for_spawn {
                    if let Some(id) = student.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            student,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(updated_students)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/status/{status}")]
async fn get_school_students_by_status(
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

    let status_str = path.into_inner();
    let status = match status_str.as_str() {
        "active" => StudentStatus::Active,
        "suspended" => StudentStatus::Suspended,
        "graduated" => StudentStatus::Graduated,
        "left" => StudentStatus::Left,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid status. Must be 'active', 'suspended', 'graduated', or 'left'"
                    .to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_school_students_by_status(&IdType::from_string(claims.id.clone()), status)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/students")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // =============================================
            // PUBLIC ROUTES (READ-ONLY)
            // =============================================
            // Student Listing & Retrieval
            .service(get_all_students) // GET    /school/students - Get all students with optional filtering and pagination
            .service(get_all_students_with_details) // GET    /school/students/with-details - Get all students with user, school, and class relations
            .service(get_active_students) // GET    /school/students/active - Get only active students
            .service(get_student_by_id) // GET    /school/students/{id} - Get student by ID
            .service(get_student_by_id_with_details) // GET    /school/students/{id}/with-details - Get student by ID with relations
            .service(get_student_by_user_id) // GET    /school/students/user/{user_id} - Get student by associated user ID
            .service(get_student_by_email) // GET    /school/students/email/{email} - Get student by email address
            .service(get_student_by_registration_number) // GET    /school/students/registration/{registration_number} - Get student by registration number
            .service(get_students_by_creator_id) // GET    /school/students/creator/{creator_id} - Get students created by specific user
            .service(get_students_by_class_id) // GET    /school/students/class/{creator_id} - Get students  by class id
            .service(get_students_by_status) // GET    /school/students/status/{status} - Get students by status (active/suspended/graduated/left)
            .service(get_class_roster) // GET    /school/students/class/{class_id}/roster - Get class roster with student details
            .service(get_school_students_by_status) // GET    /school/students/status/{status} - Get students by status
            // Permission Checking
            .service(is_user_student_of_school) // GET    /school/students/check/{user_id} - Check if user is student of current school
            .service(is_student_in_class) // GET    /school/students/check/{student_id}/class/{class_id} - Check if student is in specific class
            // Statistics & Analytics
            .service(count_students) // GET    /school/students/stats/count - Get total count of students in school
            .service(count_students_by_creator_id) // GET    /school/students/stats/count-by-creator/{creator_id} - Count students created by specific user
            .service(count_students_by_status) // GET    /school/students/stats/count-by-status/{status} - Count students by status in school
            .service(count_students_by_class_id) // GET    /school/students/stats/count-by-class/{class_id} - Count students in specific class
            .service(get_school_student_statistics) // GET    /school/students/stats/school-statistics - Get detailed student statistics for school
            // =============================================
            // PROTECTED ROUTES (WRITE OPERATIONS)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            // =============================================
            // Single Student Operations
            .service(create_student) // POST   /school/students - Create new student
            .service(update_student) // PUT    /school/students/{id} - Update student by ID
            .service(delete_student) // DELETE /school/students/{id} - Delete student by ID
            // Student Status Management
            .service(suspend_student) // PUT    /school/students/{id}/suspend - Suspend a student
            .service(activate_student) // PUT    /school/students/{id}/activate - Activate a student
            .service(graduate_student) // PUT    /school/students/{id}/graduate - Graduate a student
            // =============================================
            // BULK OPERATIONS
            // =============================================
            // Bulk Creation
            .service(create_many_students) // POST   /school/students/bulk - Create multiple students
            // Bulk Updates
            .service(update_many_students) // PUT    /school/students/bulk-update - Update multiple students in single request
            .service(bulk_update_student_status) // PUT    /school/students/bulk/status - Bulk update status for multiple students
            .service(bulk_add_tags_to_students) // PUT    /school/students/bulk/add-tags - Bulk add tags to multiple students
            .service(bulk_remove_tags_from_students) // PUT    /school/students/bulk/remove-tags - Bulk remove tags from multiple students
            .service(transfer_students_to_class) // PUT    /school/students/bulk/transfer-class - Transfer students to another class
            // Bulk Deletion
            .service(delete_many_students), // DELETE /school/students/bulk - Delete multiple students by IDs
    );
}
