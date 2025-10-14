use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::student::{
        BulkStudentIds, BulkStudentTags, BulkUpdateStudentStatus, PrepareStudentRequest,
        PrepareStudentsBulkRequest, Student, StudentStatus, UpdateStudent,
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::student_repo::StudentRepo,
    services::{event_service::EventService, student_service::StudentService},
};

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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

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

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .create_student_with_events(student_data, &state)
        .await
    {
        Ok(student) => HttpResponse::Created().json(student),
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
    match service
        .update_student_with_events(&student_id, data.into_inner(), &state)
        .await
    {
        Ok(student) => HttpResponse::Ok().json(student),
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
async fn count_students(req: actix_web::HttpRequest, state: web::Data<AppState>) -> impl Responder {
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
        .count_students_by_school_id(&IdType::from_string(claims.id.clone()))
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

    match service.get_all_students_with_relations().await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
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
    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service.get_student_by_id_with_relations(&student_id).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
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

    match service.get_students_by_class_id(&class_id).await {
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

#[get("/school/{school_id}/class/{class_id}")]
async fn get_students_by_school_and_class(
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

    let (school_id_str, class_id_str) = path.into_inner();
    let school_id = IdType::from_string(school_id_str);
    let class_id = IdType::from_string(class_id_str);

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_students_by_school_and_class(&school_id, &class_id)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[put("/{id}/merged")]
async fn update_student_merged(
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

    match service
        .update_student_merged(&student_id, data.into_inner())
        .await
    {
        Ok(student) => {
            // Broadcast updated student event
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

/// Create multiple students with validation for school
#[post("/bulk/validation")]
async fn create_many_students_with_validation(
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

    match service
        .create_many_students_with_validation(students_with_school)
        .await
    {
        Ok(students) => {
            let state_clone = state.clone();
            let students_for_spawn = students.clone();

            actix_rt::spawn(async move {
                for student in &students_for_spawn {
                    if let Some(id) = student.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            student,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(students)
        }
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

    match service.get_class_roster(&class_id).await {
        Ok(roster) => HttpResponse::Ok().json(roster),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get students by admission year
#[get("/admission-year/{year}")]
async fn get_students_by_admission_year(
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

    let year_str = path.into_inner();
    let year = match year_str.parse::<i32>() {
        Ok(year) => year,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid admission year".to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = StudentRepo::new(&school_db);
    let service = StudentService::new(&repo);

    match service
        .get_students_by_admission_year(year, Some(&IdType::from_string(claims.id.clone())))
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
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

/// Prepare students for bulk creation
#[post("/bulk/prepare")]
async fn prepare_students_for_bulk_creation(
    req: actix_web::HttpRequest,
    data: web::Json<PrepareStudentRequest>,
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

    match service.prepare_students(&data.into_inner()).await {
        Ok(prepared_students) => HttpResponse::Ok().json(prepared_students),
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

#[post("/bulk/prepare-custom")]
async fn prepare_students_for_bulk_creation_custom(
    req: actix_web::HttpRequest,
    data: web::Json<PrepareStudentsBulkRequest>,
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

    // Use provided school_id or fall back to token's school_id
    let school_id = if let Some(school_id_str) = &data.school_id {
        match ObjectId::from_str(school_id_str) {
            Ok(id) => Some(id),
            Err(e) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: format!("Invalid school_id: {}", e),
                });
            }
        }
    } else {
        // Use school_id from token
        match ObjectId::from_str(&claims.id) {
            Ok(id) => Some(id),
            Err(e) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: format!("Failed to parse school id from token: {}", e),
                });
            }
        }
    };

    // Parse optional class_id
    let class_id = if let Some(class_id_str) = &data.class_id {
        match ObjectId::from_str(class_id_str) {
            Ok(id) => Some(id),
            Err(e) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: format!("Invalid class_id: {}", e),
                });
            }
        }
    } else {
        None
    };

    // Parse optional creator_id
    let creator_id = if let Some(creator_id_str) = &data.creator_id {
        match ObjectId::from_str(creator_id_str) {
            Ok(id) => Some(id),
            Err(e) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: format!("Invalid creator_id: {}", e),
                });
            }
        }
    } else {
        None
    };

    match service.prepare_students_for_bulk_creation(
        data.students.clone(),
        school_id,
        class_id,
        creator_id,
    ) {
        Ok(prepared_students) => HttpResponse::Ok().json(prepared_students),
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
            .service(get_students_by_school_and_class) // GET    /school/students/school/{school_id}/class/{class_id} - Get students by school and class combination
            .service(get_class_roster) // GET    /school/students/class/{class_id}/roster - Get class roster with student details
            .service(get_students_by_admission_year) // GET    /school/students/admission-year/{year} - Get students by admission year
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
            // =============================================
            // Single Student Operations
            .service(create_student) // POST   /school/students - Create new student
            .service(update_student) // PUT    /school/students/{id} - Update student by ID
            .service(update_student_merged) // PUT    /school/students/{id}/merged - Update student with full data merge
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
            .service(create_many_students_with_validation) // POST   /school/students/bulk/validation - Create multiple students with comprehensive validation
            .service(prepare_students_for_bulk_creation) // POST   /school/students/bulk/prepare - Prepare student data for bulk creation
            .service(prepare_students_for_bulk_creation) // POST /school/students/bulk/prepare - The endpoint returns the prepared students with the specified school_id, class_id, and
            .service(prepare_students_for_bulk_creation_custom) // POST /school/students/bulk/prepar
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
