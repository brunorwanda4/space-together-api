use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::teacher::{
        BulkTeacherIds, BulkTeacherTags, BulkUpdateTeacherActive, PrepareTeacherRequest, Teacher,
        TeacherCountQuery, TeacherType, UpdateTeacher,
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::teacher_repo::TeacherRepo,
    services::{event_service::EventService, teacher_service::TeacherService},
};

#[get("")]
async fn get_all_teachers(
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

    match service.get_active_teachers().await {
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

#[post("")]
async fn create_teacher(
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

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

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .create_teacher_with_events(teacher_data, &state)
        .await
    {
        Ok(teacher) => HttpResponse::Created().json(teacher),
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
    match service
        .update_teacher_with_events(&teacher_id, data.into_inner(), &state)
        .await
    {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .count_teachers_by_school_id(
            &IdType::from_string(claims.id.clone()),
            query.gender.clone(),
            query.teacher_type,
        )
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-details")]
async fn get_all_teachers_with_details(
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

    match service.get_all_teachers_with_relations().await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-details")]
async fn get_teacher_by_id_with_details(
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

    match service.get_teacher_by_id_with_relations(&teacher_id).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

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

    match service.get_teachers_by_class_id(&class_id).await {
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

#[get("/type/{type}")]
async fn get_teachers_by_type(
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.get_teachers_by_type(teacher_type).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}/type/{type}")]
async fn get_teachers_by_school_and_type(
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .get_teachers_by_school_and_type(&school_id, teacher_type)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[put("/{id}/merged")]
async fn update_teacher_merged(
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

    match service
        .update_teacher_merged(&teacher_id, data.into_inner())
        .await
    {
        Ok(teacher) => {
            // Broadcast updated teacher event
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.count_teachers_by_type(teacher_type).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

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

/// Create multiple teachers for school
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

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

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .create_many_teachers_with_events(teachers_with_school, &state)
        .await
    {
        Ok(teachers) => HttpResponse::Created().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple teachers with validation for school
#[post("/bulk/validation")]
async fn create_many_teachers_with_validation(
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

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

    match service
        .create_many_teachers_with_validation(teachers_with_school)
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
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update active status for multiple teachers
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.bulk_update_active(&data.into_inner()).await {
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
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk add tags to multiple teachers
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.bulk_add_tags(&data.into_inner()).await {
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
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk remove tags from multiple teachers
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.bulk_remove_tags(&data.into_inner()).await {
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
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Delete multiple teachers
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

/// Check if user is teacher of this school
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
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .is_user_teacher_of_school(&user_id, &IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(is_teacher) => HttpResponse::Ok().json(serde_json::json!({ "is_teacher": is_teacher })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if teacher is in a specific class
#[get("/check/{teacher_id}/class/{class_id}")]
async fn is_teacher_in_class(
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

    let (teacher_id_str, class_id_str) = path.into_inner();
    let teacher_id = IdType::from_string(teacher_id_str);
    let class_id = IdType::from_string(class_id_str);

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service.is_teacher_in_class(&teacher_id, &class_id).await {
        Ok(in_class) => HttpResponse::Ok().json(serde_json::json!({ "in_class": in_class })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if teacher teaches a specific subject
#[get("/check/{teacher_id}/subject/{subject_id}")]
async fn is_teacher_in_subject(
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

    let (teacher_id_str, subject_id_str) = path.into_inner();
    let teacher_id = IdType::from_string(teacher_id_str);
    let subject_id = IdType::from_string(subject_id_str);

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .is_teacher_in_subject(&teacher_id, &subject_id)
        .await
    {
        Ok(in_subject) => HttpResponse::Ok().json(serde_json::json!({ "in_subject": in_subject })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get class teachers with details
#[get("/class/{class_id}/teachers")]
async fn get_class_teachers(
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

    match service.get_class_teachers(&class_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get subject teachers with details
#[get("/subject/{subject_id}/teachers")]
async fn get_subject_teachers(
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

    match service.get_subject_teachers(&subject_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Activate a teacher
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

/// Deactivate a teacher
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

/// Get teacher statistics for a school
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .get_school_teacher_statistics(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get head teachers for a school
#[get("/head-teachers")]
async fn get_school_head_teachers(
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
        .get_school_head_teachers(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get subject teachers for a school
#[get("/subject-teachers")]
async fn get_school_subject_teachers(
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
        .get_school_subject_teachers(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get deputy teachers for a school
#[get("/deputy-teachers")]
async fn get_school_deputy_teachers(
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
        .get_school_deputy_teachers(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get regular teachers for a school
#[get("/regular-teachers")]
async fn get_school_regular_teachers(
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
        .get_school_regular_teachers(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Add classes to teacher
#[put("/{id}/add-classes")]
async fn add_classes_to_teacher(
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

    match service.add_classes_to_teacher(&teacher_id, class_ids).await {
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

/// Add subjects to teacher
#[put("/{id}/add-subjects")]
async fn add_subjects_to_teacher(
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
        .add_subjects_to_teacher(&teacher_id, subject_ids)
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

/// Find teachers by name pattern
#[get("/search/{name_pattern}")]
async fn find_teachers_by_name_pattern(
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

    let name_pattern = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .find_teachers_by_name_pattern(&name_pattern, Some(&IdType::from_string(claims.id.clone())))
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
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

/// Bulk update multiple teachers
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    // Convert the request data to the format expected by the service
    let updates: Vec<(IdType, UpdateTeacher)> = data
        .into_inner()
        .into_iter()
        .map(|(id_str, update)| (IdType::from_string(id_str), update))
        .collect();

    match service.update_many_teachers(updates).await {
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
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare teachers for bulk creation with custom parameters
#[post("/bulk/prepare-custom")]
async fn prepare_teachers_for_bulk_creation_custom(
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

    match service.prepare_teachers_for_bulk_creation(data.teachers.clone(), school_id, creator_id) {
        Ok(prepared_teachers) => HttpResponse::Ok().json(prepared_teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get teachers by type for current school
#[get("/type/{type}/school")]
async fn get_school_teachers_by_type(
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

    let school_db = state.db.get_db(&claims.database_name);
    let repo = TeacherRepo::new(&school_db);
    let service = TeacherService::new(&repo);

    match service
        .get_school_teachers_by_type(&IdType::from_string(claims.id.clone()), teacher_type)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/teachers")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // =============================================
            // PUBLIC ROUTES (READ-ONLY)
            // =============================================
            // Teacher Listing & Retrieval
            .service(get_all_teachers) // GET    /school/teachers - Get all teachers with optional filtering and pagination
            .service(get_all_teachers_with_details) // GET    /school/teachers/with-details - Get all teachers with user, school, classes, and subjects relations
            .service(get_active_teachers) // GET    /school/teachers/active - Get only active teachers
            .service(get_teacher_by_id) // GET    /school/teachers/{id} - Get teacher by ID
            .service(get_teacher_by_id_with_details) // GET    /school/teachers/{id}/with-details - Get teacher by ID with relations
            .service(get_teacher_by_user_id) // GET    /school/teachers/user/{user_id} - Get teacher by associated user ID
            .service(get_teacher_by_email) // GET    /school/teachers/email/{email} - Get teacher by email address
            .service(get_teachers_by_creator_id) // GET    /school/teachers/creator/{creator_id} - Get teachers created by specific user
            .service(get_teachers_by_class_id) // GET    /school/teachers/class/{class_id} - Get teachers by class id
            .service(get_teachers_by_subject_id) // GET    /school/teachers/subject/{subject_id} - Get teachers by subject id
            .service(get_teachers_by_type) // GET    /school/teachers/type/{type} - Get teachers by type (regular/headteacher/subjectteacher/deputy)
            .service(get_teachers_by_school_and_type) // GET    /school/teachers/school/{school_id}/type/{type} - Get teachers by school and type combination
            .service(get_class_teachers) // GET    /school/teachers/class/{class_id}/teachers - Get class teachers with details
            .service(get_subject_teachers) // GET    /school/teachers/subject/{subject_id}/teachers - Get subject teachers with details
            .service(find_teachers_by_name_pattern) // GET    /school/teachers/search/{name_pattern} - Find teachers by name pattern
            .service(get_school_teachers_by_type) // GET /school/teachers/type/{type}/school
            // Specialized Teacher Types
            .service(get_school_head_teachers) // GET    /school/teachers/head-teachers - Get head teachers for school
            .service(get_school_subject_teachers) // GET    /school/teachers/subject-teachers - Get subject teachers for school
            .service(get_school_deputy_teachers) // GET    /school/teachers/deputy-teachers - Get deputy teachers for school
            .service(get_school_regular_teachers) // GET    /school/teachers/regular-teachers - Get regular teachers for school
            // Permission Checking
            .service(is_user_teacher_of_school) // GET    /school/teachers/check/{user_id} - Check if user is teacher of current school
            .service(is_teacher_in_class) // GET    /school/teachers/check/{teacher_id}/class/{class_id} - Check if teacher is in specific class
            .service(is_teacher_in_subject) // GET    /school/teachers/check/{teacher_id}/subject/{subject_id} - Check if teacher teaches specific subject
            // Statistics & Analytics
            .service(count_teachers) // GET    /school/teachers/stats/count - Get total count of teachers in school
            .service(count_teachers_by_creator_id) // GET    /school/teachers/stats/count-by-creator/{creator_id} - Count teachers created by specific user
            .service(count_teachers_by_type) // GET    /school/teachers/stats/count-by-type/{type} - Count teachers by type in school
            .service(count_teachers_by_class_id) // GET    /school/teachers/stats/count-by-class/{class_id} - Count teachers in specific class
            .service(count_teachers_by_subject_id) // GET    /school/teachers/stats/count-by-subject/{subject_id} - Count teachers for specific subject
            .service(get_school_teacher_statistics) // GET    /school/teachers/stats/school-statistics - Get detailed teacher statistics for school
            // =============================================
            // PROTECTED ROUTES (WRITE OPERATIONS)
            // =============================================
            // Single Teacher Operations
            .service(create_teacher) // POST   /school/teachers - Create new teacher
            .service(update_teacher) // PUT    /school/teachers/{id} - Update teacher by ID
            .service(update_teacher_merged) // PUT    /school/teachers/{id}/merged - Update teacher with full data merge
            .service(delete_teacher) // DELETE /school/teachers/{id} - Delete teacher by ID
            // Teacher Status Management
            .service(activate_teacher) // PUT    /school/teachers/{id}/activate - Activate a teacher
            .service(deactivate_teacher) // PUT    /school/teachers/{id}/deactivate - Deactivate a teacher
            // Teacher-Class-Subject Relationships
            .service(add_classes_to_teacher) // PUT    /school/teachers/{id}/add-classes - Add classes to teacher
            .service(add_subjects_to_teacher) // PUT    /school/teachers/{id}/add-subjects - Add subjects to teacher
            .service(remove_classes_from_teacher) // PUT    /school/teachers/{id}/remove-classes - Remove classes from teacher
            .service(remove_subjects_from_teacher) // PUT    /school/teachers/{id}/remove-subjects - Remove subjects from teacher
            // =============================================
            // BULK OPERATIONS
            // =============================================
            // Bulk Creation
            .service(create_many_teachers) // POST   /school/teachers/bulk - Create multiple teachers
            .service(create_many_teachers_with_validation) // POST   /school/teachers/bulk/validation - Create multiple teachers with comprehensive validation
            .service(prepare_teachers_for_bulk_creation) // POST   /school/teachers/bulk/prepare - Prepare teacher data for bulk creation
            .service(prepare_teachers_for_bulk_creation_custom) // POST /school/teachers/bulk/prepare-custom
            // Bulk Updates
            .service(update_many_teachers) // PUT    /school/teachers/bulk-update - Update multiple teachers in single request
            .service(bulk_update_teacher_active) // PUT    /school/teachers/bulk/active - Bulk update active status for multiple teachers
            .service(bulk_add_tags_to_teachers) // PUT    /school/teachers/bulk/add-tags - Bulk add tags to multiple teachers
            .service(bulk_remove_tags_from_teachers) // PUT    /school/teachers/bulk/remove-tags - Bulk remove tags from multiple teachers
            // Bulk Deletion
            .service(delete_many_teachers), // DELETE /school/teachers/bulk - Delete multiple teachers by IDs
    );
}
