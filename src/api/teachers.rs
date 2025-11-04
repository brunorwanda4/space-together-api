use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        teacher::{
            BulkTeacherIds, BulkTeacherTags, BulkUpdateTeacherActive, PrepareTeacherRequest,
            Teacher, TeacherType, UpdateTeacher,
        },
    },
    guards::role_guard,
    models::{api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel},
    repositories::teacher_repo::TeacherRepo,
    services::{event_service::EventService, teacher_service::TeacherService},
};

// Request models for bulk operations
#[derive(Debug, serde::Deserialize)]
pub struct BulkTeachersRequest {
    pub teachers: Vec<Teacher>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkUpdateTeacherRequest {
    pub updates: Vec<BulkTeacherUpdateItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkTeacherUpdateItem {
    pub id: String,
    pub update: UpdateTeacher,
}

#[get("")]
async fn get_all_teachers(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service
        .get_all_teachers(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-details")]
async fn get_all_teachers_with_details(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service
        .get_all_teachers_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_teachers(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
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
async fn get_teacher_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let teacher_id = IdType::from_string(path.into_inner());

    match service.get_teacher_by_id(&teacher_id).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-details")]
async fn get_teacher_by_id_with_details(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let teacher_id = IdType::from_string(path.into_inner());

    match service.get_teacher_by_id_with_relations(&teacher_id).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/user/{user_id}")]
async fn get_teacher_by_user_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let user_id = IdType::from_string(path.into_inner());

    match service.get_teacher_by_user_id(&user_id).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/email/{email}")]
async fn get_teacher_by_email(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let email = path.into_inner();

    match service.get_teacher_by_email(&email).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/type/{type}")]
async fn get_teachers_by_type(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

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

    match service.get_teachers_by_type(teacher_type).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/class/{class_id}")]
async fn get_teachers_by_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.get_teachers_by_class_id(&class_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}")]
async fn get_teachers_by_subject_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_teachers_by_subject_id(&subject_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_teachers_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.get_teachers_by_creator_id(&creator_id).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_teacher(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Teacher>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only admin, staff, or teachers can create teachers
    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.create_teacher(data.into_inner()).await {
        Ok(teacher) => {
            // Broadcast created teacher event
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_teacher_id_str = path.into_inner();

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &target_teacher_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_teacher_id = IdType::from_string(target_teacher_id_str);
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service
        .update_teacher(&target_teacher_id, data.into_inner())
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

#[put("/{id}/merged")]
async fn update_teacher_merged(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateTeacher>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_teacher_id_str = path.into_inner();

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &target_teacher_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_teacher_id = IdType::from_string(target_teacher_id_str);
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service
        .update_teacher_merged(&target_teacher_id, data.into_inner())
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

#[delete("/{id}")]
async fn delete_teacher(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let target_teacher_id_str = path.into_inner();

    // Only admin or teacher creator can delete teachers
    if let Err(err) = crate::guards::role_guard::check_admin_or_teacher_creator(
        &logged_user,
        &target_teacher_id_str,
    ) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_teacher_id = IdType::from_string(target_teacher_id_str);
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    // Get teacher before deletion for broadcasting
    let teacher_before_delete = repo.find_by_id(&target_teacher_id).await.ok().flatten();

    match service.delete_teacher(&target_teacher_id).await {
        Ok(_) => {
            // Broadcast deleted teacher event
            if let Some(teacher) = teacher_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = teacher.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "teacher",
                            &id.to_hex(),
                            &teacher,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Teacher deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count")]
async fn count_teachers(state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());

    // Use doc! {} instead of None for count_documents
    match repo.collection.count_documents(doc! {}).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: format!("Failed to count teachers: {}", e),
        }),
    }
}

#[get("/stats/count-by-type/{type}")]
async fn count_teachers_by_type(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

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

    match service.count_teachers_by_type(teacher_type).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-class/{class_id}")]
async fn count_teachers_by_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.count_teachers_by_class_id(&class_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-subject/{subject_id}")]
async fn count_teachers_by_subject_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.count_teachers_by_subject_id(&subject_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_teachers_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.count_teachers_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple teachers
#[post("/bulk")]
async fn create_many_teachers(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkTeachersRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.create_many_teachers(data.teachers.clone()).await {
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

/// Create multiple teachers with validation
#[post("/bulk/validation")]
async fn create_many_teachers_with_validation(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkTeachersRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service
        .create_many_teachers_with_validation(data.teachers.clone())
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

/// Bulk update multiple teachers
#[put("/bulk")]
async fn update_many_teachers(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateTeacherRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let updates: Vec<(IdType, UpdateTeacher)> = data
        .updates
        .iter()
        .map(|item| {
            let id = IdType::from_string(item.id.clone());
            (id, item.update.clone())
        })
        .collect();

    // Check permissions for each teacher update
    for (id, _) in &updates {
        if let Err(err) =
            crate::guards::role_guard::check_teacher_access(&logged_user, &id.as_string())
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update teacher: {}", err)
            }));
        }
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.update_many_teachers(updates).await {
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

/// Bulk update active status for multiple teachers
#[put("/bulk/active")]
async fn bulk_update_teacher_active(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkUpdateTeacherActive>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkTeacherTags>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkTeacherTags>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkTeacherIds>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.delete_many_teachers(&data.into_inner()).await {
        Ok(deleted_count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully deleted {} teachers", deleted_count)
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare teachers for bulk creation
#[post("/bulk/prepare")]
async fn prepare_teachers_for_bulk_creation(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<PrepareTeacherRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only admin, staff, or teachers can prepare teachers
    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.prepare_teachers(&data.into_inner()).await {
        Ok(prepared_teachers) => HttpResponse::Ok().json(prepared_teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get class teachers with details
#[get("/class/{class_id}/teachers")]
async fn get_class_teachers(
    query: web::Query<RequestQuery>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service
        .get_class_teachers(&class_id, query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get subject teachers with details
#[get("/subject/{subject_id}/teachers")]
async fn get_subject_teachers(
    query: web::Query<RequestQuery>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service
        .get_subject_teachers(&subject_id, query.filter.clone(), query.limit, query.skip)
        .await
    {
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &teacher_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &teacher_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
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

/// Add classes to teacher
#[put("/{id}/add-classes")]
async fn add_classes_to_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &teacher_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    // Convert string IDs to ObjectIds
    use mongodb::bson::oid::ObjectId;
    use std::str::FromStr;

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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &teacher_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    // Convert string IDs to ObjectIds
    use mongodb::bson::oid::ObjectId;
    use std::str::FromStr;

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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &teacher_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    // Convert string IDs to ObjectIds
    use mongodb::bson::oid::ObjectId;
    use std::str::FromStr;

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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let teacher_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update teacher
    if let Err(err) =
        crate::guards::role_guard::check_teacher_access(&logged_user, &teacher_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    // Convert string IDs to ObjectIds
    use mongodb::bson::oid::ObjectId;
    use std::str::FromStr;

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
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    let name_pattern = path.into_inner();

    match service
        .find_teachers_by_name_pattern(&name_pattern, None)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get teacher statistics
#[get("/stats/statistics")]
async fn get_teacher_statistics(state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    // Since we don't have school relationships, we'll get global statistics
    // You might want to implement a method for global teacher statistics
    match service.get_all_teachers(None, None, None).await {
        Ok(teachers) => {
            let mut stats = std::collections::HashMap::new();
            for teacher in teachers {
                *stats.entry(teacher.r#type).or_insert(0) += 1;
            }
            HttpResponse::Ok().json(stats)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get head teachers
#[get("/head-teachers")]
async fn get_head_teachers(state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.get_teachers_by_type(TeacherType::HeadTeacher).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get subject teachers
// Change this endpoint name to avoid conflict
#[get("/subject-teachers-list")]
async fn get_all_subject_teachers(state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service
        .get_teachers_by_type(TeacherType::SubjectTeacher)
        .await
    {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get deputy teachers
#[get("/deputy-teachers")]
async fn get_deputy_teachers(state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.get_teachers_by_type(TeacherType::Deputy).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get regular teachers
#[get("/regular-teachers")]
async fn get_regular_teachers(state: web::Data<AppState>) -> impl Responder {
    let repo = TeacherRepo::new(&state.db.main_db());
    let service = TeacherService::new(&repo);

    match service.get_teachers_by_type(TeacherType::Regular).await {
        Ok(teachers) => HttpResponse::Ok().json(teachers),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/teachers")
            // Public routes (read-only)
            .service(get_all_teachers) // GET /teachers - Get all teachers with optional filtering and pagination
            .service(get_all_teachers_with_details) // GET /teachers/with-details - Get all teachers with user, classes, and subjects relations
            .service(get_active_teachers) // GET /teachers/active - Get only active teachers
            .service(get_teacher_by_id) // GET /teachers/{id} - Get teacher by ID
            .service(get_teacher_by_id_with_details) // GET /teachers/{id}/with-details - Get teacher by ID with relations
            .service(get_teacher_by_user_id) // GET /teachers/user/{user_id} - Get teacher by associated user ID
            .service(get_teacher_by_email) // GET /teachers/email/{email} - Get teacher by email address
            .service(get_teachers_by_type) // GET /teachers/type/{type} - Get teachers by type (regular/headteacher/subjectteacher/deputy)
            .service(get_teachers_by_class_id) // GET /teachers/class/{class_id} - Get teachers by class ID
            .service(get_teachers_by_subject_id) // GET /teachers/subject/{subject_id} - Get teachers by subject ID
            .service(get_teachers_by_creator_id) // GET /teachers/creator/{creator_id} - Get teachers by creator ID
            .service(get_class_teachers) // GET /teachers/class/{class_id}/teachers - Get class teachers with details
            .service(get_subject_teachers) // GET /teachers/subject/{subject_id}/teachers - Get subject teachers with details
            .service(find_teachers_by_name_pattern) // GET /teachers/search/{name_pattern} - Find teachers by name pattern
            // Specialized teacher types
            .service(get_head_teachers) // GET /teachers/head-teachers - Get head teachers
            .service(get_all_subject_teachers) // GET /teachers/subject-teachers-list - Get subject teachers
            .service(get_deputy_teachers) // GET /teachers/deputy-teachers - Get deputy teachers
            .service(get_regular_teachers) // GET /teachers/regular-teachers - Get regular teachers
            // Statistics & Analytics
            .service(count_teachers) // GET /teachers/stats/count - Count all teachers
            .service(count_teachers_by_type) // GET /teachers/stats/count-by-type/{type} - Count teachers by type
            .service(count_teachers_by_class_id) // GET /teachers/stats/count-by-class/{class_id} - Count teachers by class ID
            .service(count_teachers_by_subject_id) // GET /teachers/stats/count-by-subject/{subject_id} - Count teachers by subject ID
            .service(count_teachers_by_creator_id) // GET /teachers/stats/count-by-creator/{creator_id} - Count teachers by creator ID
            .service(get_teacher_statistics) // GET /teachers/stats/statistics - Get detailed teacher statistics
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_teacher) // POST /teachers - Create new teacher (Admin/Staff/Teacher only)
            .service(update_teacher) // PUT /teachers/{id} - Update teacher (Admin/Teacher access only)
            .service(update_teacher_merged) // PUT /teachers/{id}/merged - Update teacher with merge (Admin/Teacher access only)
            .service(delete_teacher) // DELETE /teachers/{id} - Delete teacher (Admin/Teacher creator only)
            // Teacher Status Management
            .service(activate_teacher) // PUT /teachers/{id}/activate - Activate a teacher
            .service(deactivate_teacher) // PUT /teachers/{id}/deactivate - Deactivate a teacher
            // Teacher-Class-Subject Relationships
            .service(add_classes_to_teacher) // PUT /teachers/{id}/add-classes - Add classes to teacher
            .service(add_subjects_to_teacher) // PUT /teachers/{id}/add-subjects - Add subjects to teacher
            .service(remove_classes_from_teacher) // PUT /teachers/{id}/remove-classes - Remove classes from teacher
            .service(remove_subjects_from_teacher) // PUT /teachers/{id}/remove-subjects - Remove subjects from teacher
            // Bulk operations (protected)
            .service(create_many_teachers) // POST /teachers/bulk - Create multiple teachers
            .service(create_many_teachers_with_validation) // POST /teachers/bulk/validation - Create multiple teachers with validation
            .service(update_many_teachers) // PUT /teachers/bulk - Update multiple teachers
            .service(bulk_update_teacher_active) // PUT /teachers/bulk/active - Bulk update active status for multiple teachers
            .service(bulk_add_tags_to_teachers) // PUT /teachers/bulk/add-tags - Bulk add tags to multiple teachers
            .service(bulk_remove_tags_from_teachers) // PUT /teachers/bulk/remove-tags - Bulk remove tags from multiple teachers
            .service(delete_many_teachers) // DELETE /teachers/bulk - Delete multiple teachers by IDs
            .service(prepare_teachers_for_bulk_creation), // POST /teachers/bulk/prepare - Prepare teachers for bulk creation
    );
}
