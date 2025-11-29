use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        student::{
            BulkStudentIds, BulkStudentTags, BulkUpdateStudentStatus, Student, StudentCountQuery,
            StudentStatus, UpdateStudent,
        },
    },
    guards::role_guard,
    models::{api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel},
    repositories::student_repo::StudentRepo,
    services::{event_service::EventService, student_service::StudentService},
};

// Request models for bulk operations
#[derive(Debug, serde::Deserialize)]
pub struct BulkStudentsRequest {
    pub students: Vec<Student>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkUpdateStudentRequest {
    pub updates: Vec<BulkStudentUpdateItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkStudentUpdateItem {
    pub id: String,
    pub update: UpdateStudent,
}

#[get("")]
async fn get_all_students(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service
        .get_all_students(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-details")]
async fn get_all_students_with_details(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service
        .get_all_students_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_students(state: web::Data<AppState>) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service.get_active_students().await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_student_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let student_id = IdType::from_string(path.into_inner());

    match service.get_student_by_id(&student_id).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-details")]
async fn get_student_by_id_with_details(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let student_id = IdType::from_string(path.into_inner());

    match service.get_student_by_id_with_relations(&student_id).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/user/{user_id}")]
async fn get_student_by_user_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let user_id = IdType::from_string(path.into_inner());

    match service.get_student_by_user_id(&user_id).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/email/{email}")]
async fn get_student_by_email(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let email = path.into_inner();

    match service.get_student_by_email(&email).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/registration/{registration_number}")]
async fn get_student_by_registration_number(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let registration_number = path.into_inner();

    match service
        .get_student_by_registration_number(&registration_number)
        .await
    {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}")]
async fn get_students_by_school_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let school_id = IdType::from_string(path.into_inner());

    match service.get_students_by_school_id(&school_id).await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/class/{class_id}")]
async fn get_students_by_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service
        .get_students_by_class_id(&class_id, query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_students_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.get_students_by_creator_id(&creator_id).await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/status/{status}")]
async fn get_students_by_status(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

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

    match service.get_students_by_status(status).await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}/status/{status}")]
async fn get_school_students_by_status(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let (school_id_str, status_str) = path.into_inner();
    let school_id = IdType::from_string(school_id_str);

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

    match service
        .get_school_students_by_status(&school_id, status)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_student(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Student>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only admin, staff, or teachers can create students
    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service.create_student(data.into_inner()).await {
        Ok(student) => {
            // Broadcast created student event
            let student_clone = student.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = student_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let target_student_id_str = path.into_inner();

    // Check if user has permission to update student
    if let Err(err) =
        crate::guards::role_guard::check_student_access(&logged_user, &target_student_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_student_id = IdType::from_string(target_student_id_str);
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service
        .update_student(&target_student_id, data.into_inner())
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

#[delete("/{id}")]
async fn delete_student(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let target_student_id_str = path.into_inner();

    // Only admin or student creator can delete students
    if let Err(err) = crate::guards::role_guard::check_admin_or_student_creator(
        &logged_user,
        &target_student_id_str,
    ) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_student_id = IdType::from_string(target_student_id_str);
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    // Get student before deletion for broadcasting
    let student_before_delete = repo.find_by_id(&target_student_id).await.ok().flatten();

    match service.delete_student(&target_student_id).await {
        Ok(_) => {
            // Broadcast deleted student event
            if let Some(student) = student_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = student.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "student",
                            &id.to_hex(),
                            &student,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Student deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-school/{school_id}")]
async fn count_students_by_school_id(
    query: web::Query<StudentCountQuery>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let school_id = IdType::from_string(path.into_inner());

    match service
        .count_students_by_school_id(&school_id, query.gender.clone(), query.status)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-class/{class_id}")]
async fn count_students_by_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.count_students_by_class_id(&class_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_students_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.count_students_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-status/{status}")]
async fn count_students_by_status(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

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

    match service.count_students_by_status(status).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple students
#[post("/bulk")]
async fn create_many_students(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkStudentsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service.create_many_students(data.students.clone()).await {
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

/// Bulk update multiple students
#[put("/bulk")]
async fn update_many_students(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateStudentRequest>,
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

    let updates: Vec<(IdType, UpdateStudent)> = data
        .updates
        .iter()
        .map(|item| {
            let id = IdType::from_string(item.id.clone());
            (id, item.update.clone())
        })
        .collect();

    // Check permissions for each student update
    for (id, _) in &updates {
        if let Err(err) =
            crate::guards::role_guard::check_student_access(&logged_user, &id.as_string())
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update student: {}", err)
            }));
        }
    }

    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    match service.update_many_students(updates).await {
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

/// Bulk update status for multiple students
#[put("/bulk/status")]
async fn bulk_update_student_status(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkUpdateStudentStatus>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkStudentTags>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkStudentTags>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkStudentIds>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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
    user: web::ReqData<AuthUserDto>,
    data: web::Json<(BulkStudentIds, String)>, // (student_ids, new_class_id)
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let (student_ids, new_class_id_str) = data.into_inner();
    let new_class_id = IdType::from_string(new_class_id_str);

    let repo = StudentRepo::new(&state.db.main_db());
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

/// Get class roster with student details
#[get("/class/{class_id}/roster")]
async fn get_class_roster(
    query: web::Query<RequestQuery>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service
        .get_class_roster(&class_id, query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(roster) => HttpResponse::Ok().json(roster),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get student statistics for a school
#[get("/stats/school/{school_id}/statistics")]
async fn get_school_student_statistics(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = StudentRepo::new(&state.db.main_db());
    let service = StudentService::new(&repo);

    let school_id = IdType::from_string(path.into_inner());

    match service.get_school_student_statistics(&school_id).await {
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let student_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update student
    if let Err(err) =
        crate::guards::role_guard::check_student_access(&logged_user, &student_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let student_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update student
    if let Err(err) =
        crate::guards::role_guard::check_student_access(&logged_user, &student_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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
    let logged_user = match req.extensions().get::<AuthUserDto>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            }))
        }
    };

    let student_id = IdType::from_string(path.into_inner());

    // Check if user has permission to update student
    if let Err(err) =
        crate::guards::role_guard::check_student_access(&logged_user, &student_id.as_string())
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = StudentRepo::new(&state.db.main_db());
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

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/students")
            // Public routes (read-only)
            .service(get_all_students) // GET /students - Get all students with optional filtering and pagination
            .service(get_all_students_with_details) // GET /students/with-details - Get all students with user, school, and class relations
            .service(get_active_students) // GET /students/active - Get only active students
            .service(get_student_by_id) // GET /students/{id} - Get student by ID
            .service(get_student_by_id_with_details) // GET /students/{id}/with-details - Get student by ID with relations
            .service(get_student_by_user_id) // GET /students/user/{user_id} - Get student by associated user ID
            .service(get_student_by_email) // GET /students/email/{email} - Get student by email address
            .service(get_student_by_registration_number) // GET /students/registration/{registration_number} - Get student by registration number
            .service(get_students_by_school_id) // GET /students/school/{school_id} - Get students by school ID
            .service(get_students_by_class_id) // GET /students/class/{class_id} - Get students by class ID
            .service(get_students_by_creator_id) // GET /students/creator/{creator_id} - Get students by creator ID
            .service(get_students_by_status) // GET /students/status/{status} - Get students by status (active/suspended/graduated/left)
            .service(get_school_students_by_status) // GET /students/school/{school_id}/status/{status} - Get students by school and status
            .service(get_class_roster) // GET /students/class/{class_id}/roster - Get class roster with student details
            // Statistics & Analytics
            .service(count_students_by_school_id) // GET /students/stats/count-by-school/{school_id} - Count students by school ID
            .service(count_students_by_class_id) // GET /students/stats/count-by-class/{class_id} - Count students by class ID
            .service(count_students_by_creator_id) // GET /students/stats/count-by-creator/{creator_id} - Count students by creator ID
            .service(count_students_by_status) // GET /students/stats/count-by-status/{status} - Count students by status
            .service(get_school_student_statistics) // GET /students/stats/school/{school_id}/statistics - Get detailed student statistics for school
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_student) // POST /students - Create new student (Admin/Staff/Teacher only)
            .service(update_student) // PUT /students/{id} - Update student (Admin/Student access only)
            .service(delete_student) // DELETE /students/{id} - Delete student (Admin/Student creator only)
            // Student Status Management
            .service(suspend_student) // PUT /students/{id}/suspend - Suspend a student
            .service(activate_student) // PUT /students/{id}/activate - Activate a student
            .service(graduate_student) // PUT /students/{id}/graduate - Graduate a student
            // Bulk operations (protected)
            .service(create_many_students) // POST /students/bulk - Create multiple students
            .service(update_many_students) // PUT /students/bulk - Update multiple students
            .service(bulk_update_student_status) // PUT /students/bulk/status - Bulk update status for multiple students
            .service(bulk_add_tags_to_students) // PUT /students/bulk/add-tags - Bulk add tags to multiple students
            .service(bulk_remove_tags_from_students) // PUT /students/bulk/remove-tags - Bulk remove tags from multiple students
            .service(transfer_students_to_class) // PUT /students/bulk/transfer-class - Transfer students to another class
            .service(delete_many_students), // DELETE /students/bulk - Delete multiple students by IDs
    );
}
