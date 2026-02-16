use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        assignment::{Assignment, AssignmentPartial, Submission, SubmissionPartial},
        auth_user::AuthUserDto,
        common_details::UserRole,
    },
    guards::role_guard::{require_permission, require_feature_enabled, require_parent_child_access},
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        assignment_service::AssignmentService, 
        event_service::EventService,
        role_service::RoleService,
        feature_service::FeatureService,
        parent_service::ParentService,
    },
    utils::{api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value},
};

// =========================
// ASSIGNMENT ENDPOINTS
// =========================

#[get("")]
async fn get_all_assignments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all_assignments_with_relations(
            query.filter.clone(),
            query.limit,
            query.skip,
            extra_match,
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_assignment_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    match service
        .find_one_assignment_with_relations(Some(&id), None)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Assignment>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    
    // Check if assignments feature is enabled
    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID required"
            }));
        }
    };
    
    let feature_service = FeatureService::new(&db);
    if let Err(e) = require_feature_enabled(&school_id, "assignments.enabled", &feature_service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }
    
    // Check permission: assignment.create
    let role_service = RoleService::new(&db);
    if let Err(e) = require_permission(&user, &school_id, "assignment.create", &role_service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }

    let service = AssignmentService::new(&db);

    let mut assignment = data.into_inner();

    // Set teacher_id from authenticated user if not provided
    if assignment.teacher_id.is_none() {
        match parse_object_id_value(&user.id) {
            Ok(teacher_id) => {
                // Verify this user is actually a teacher
                let teacher_collection = db.collection::<mongodb::bson::Document>("teachers");
                match teacher_collection
                    .find_one(doc! { "user_id": teacher_id })
                    .await
                {
                    Ok(Some(teacher_doc)) => {
                        if let Ok(teacher_oid) = teacher_doc.get_object_id("_id") {
                            assignment.teacher_id = Some(teacher_oid);
                        }
                    }
                    _ => {
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "message": "Teacher record not found for this user"
                        }));
                    }
                }
            }
            Err(err) => return HttpResponse::BadRequest().json(err),
        }
    }

    match service.create_assignment(assignment).await {
        Ok(assignment) => {
            let assignment_clone = assignment.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = assignment_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "assignment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &assignment_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(assignment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<AssignmentPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    
    // Check permission: assignment.update
    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID required"
            }));
        }
    };
    
    let role_service = RoleService::new(&db);
    if let Err(e) = require_permission(&user, &school_id, "assignment.update", &role_service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }
    
    let service = AssignmentService::new(&db);

    // Verify user is the assignment creator or admin
    if !matches!(user.role, Some(UserRole::ADMIN)) {
        match service.find_one_assignment(Some(&id), None).await {
            Ok(assignment) => {
                if let (Some(teacher_id), Ok(user_oid)) =
                    (assignment.teacher_id, parse_object_id_value(&user.id))
                {
                    let teacher_collection = db.collection::<mongodb::bson::Document>("teachers");
                    match teacher_collection.find_one(doc! { "_id": teacher_id, "user_id": user_oid })
                        .await
                    {
                        Ok(None) | Err(_) => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only update your own assignments"
                            }));
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    match service.update_assignment(&id, &data.into_inner()).await {
        Ok(assignment) => {
            let assignment_clone = assignment.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = assignment_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "assignment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &assignment_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(assignment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    
    // Check permission: assignment.delete
    let school_id = match get_school_id_from_request(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID required"
            }));
        }
    };
    
    let role_service = RoleService::new(&db);
    if let Err(e) = require_permission(&user, &school_id, "assignment.delete", &role_service).await {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": e
        }));
    }
    
    let service = AssignmentService::new(&db);

    // Only admin or assignment creator can delete
    if !matches!(user.role, Some(UserRole::ADMIN)) {
        match service.find_one_assignment(Some(&id), None).await {
            Ok(assignment) => {
                if let (Some(teacher_id), Ok(user_oid)) =
                    (assignment.teacher_id, parse_object_id_value(&user.id))
                {
                    let teacher_collection = db.collection::<mongodb::bson::Document>("teachers");
                    match teacher_collection.find_one(doc! { "_id": teacher_id, "user_id": user_oid })
                        .await
                    {
                        Ok(None) | Err(_) => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only delete your own assignments"
                            }));
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    match service.delete_assignment(&id).await {
        Ok(assignment) => {
            let assignment_clone = assignment.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = assignment_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "assignment",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &assignment_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(assignment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_assignments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_assignments(query.filter.clone(), extra_match)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// =========================
// SUBMISSION ENDPOINTS
// =========================

#[post("/{id}/submit")]
async fn submit_assignment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<Submission>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only students can submit
    if !matches!(user.role, Some(UserRole::STUDENT)) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": "Only students can submit assignments"
        }));
    }

    let assignment_id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    let mut submission = data.into_inner();

    // Set assignment_id from path
    match IdType::to_object_id(&assignment_id) {
        Ok(oid) => submission.assignment_id = Some(oid),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid assignment ID"
            }));
        }
    }

    // Set student_id from authenticated user
    match parse_object_id_value(&user.id) {
        Ok(user_oid) => {
            // Find student record
            let student_collection = db.collection::<mongodb::bson::Document>("students");
            match student_collection
                .find_one(doc! { "user_id": user_oid })
                .await
            {
                Ok(Some(student_doc)) => {
                    if let Ok(student_oid) = student_doc.get_object_id("_id") {
                        submission.student_id = Some(student_oid);

                        // Verify student belongs to the class
                        if let Ok(assignment) = service
                            .find_one_assignment(Some(&assignment_id), None)
                            .await
                        {
                            if let (Some(assignment_class_id), Some(student_class_id)) =
                                (assignment.class_id, student_doc.get_object_id("class_id").ok())
                            {
                                if assignment_class_id != student_class_id {
                                    return HttpResponse::Forbidden().json(serde_json::json!({
                                        "message": "You are not enrolled in this class"
                                    }));
                                }
                            }
                        }
                    }
                }
                _ => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Student record not found for this user"
                    }));
                }
            }
        }
        Err(err) => return HttpResponse::BadRequest().json(err),
    }

    match service.create_submission(submission).await {
        Ok(submission) => {
            let submission_clone = submission.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = submission_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "submission",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &submission_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(submission)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}/submissions")]
async fn get_assignment_submissions(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only teachers and admins can view all submissions
    if !matches!(
        user.role,
        Some(UserRole::TEACHER) | Some(UserRole::ADMIN) | Some(UserRole::SCHOOLSTAFF)
    ) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": "Insufficient permissions"
        }));
    }

    let assignment_id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    let assignment_oid = match IdType::to_object_id(&assignment_id) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid assignment ID"
            }));
        }
    };

    // Merge extra_match with assignment_id filter
    let final_match = if let Some(mut doc) = extra_match {
        doc.insert("assignment_id", assignment_oid);
        Some(doc)
    } else {
        Some(doc! { "assignment_id": assignment_oid })
    };

    match service
        .get_all_submissions_with_relations(
            query.filter.clone(),
            query.limit,
            query.skip,
            final_match,
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{assignment_id}/grade/{submission_id}")]
async fn grade_submission(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<(String, String)>,
    data: web::Json<serde_json::Value>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only teachers can grade
    if !matches!(user.role, Some(UserRole::TEACHER) | Some(UserRole::ADMIN)) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": "Only teachers can grade submissions"
        }));
    }

    let (assignment_id_str, submission_id_str) = path.into_inner();
    let assignment_id = IdType::from_string(assignment_id_str);
    let submission_id = IdType::from_string(submission_id_str);

    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    // Verify teacher is assigned to this assignment's subject
    if !matches!(user.role, Some(UserRole::ADMIN)) {
        match service
            .find_one_assignment(Some(&assignment_id), None)
            .await
        {
            Ok(assignment) => {
                if let (Some(teacher_id), Ok(user_oid)) =
                    (assignment.teacher_id, parse_object_id_value(&user.id))
                {
                    let teacher_collection = db.collection::<mongodb::bson::Document>("teachers");
                    match teacher_collection.find_one(doc! { "_id": teacher_id, "user_id": user_oid })
                        .await
                    {
                        Ok(None) | Err(_) => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only grade assignments for subjects you teach"
                            }));
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    // Extract grading data
    let score = match data.get("score").and_then(|v| v.as_f64()) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Score is required"
            }));
        }
    };

    let feedback = data
        .get("feedback")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let feedback_file = data
        .get("feedback_file")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Get teacher ObjectId
    let graded_by = match parse_object_id_value(&user.id) {
        Ok(user_oid) => {
            let teacher_collection = db.collection::<mongodb::bson::Document>("teachers");
            match teacher_collection
                .find_one(doc! { "user_id": user_oid })
                .await
            {
                Ok(Some(teacher_doc)) => match teacher_doc.get_object_id("_id") {
                    Ok(teacher_oid) => teacher_oid,
                    Err(_) => {
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "message": "Invalid teacher ID"
                        }));
                    }
                },
                _ => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Teacher record not found"
                    }));
                }
            }
        }
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    match service
        .grade_submission(&submission_id, score, feedback, feedback_file, graded_by)
        .await
    {
        Ok(submission) => {
            let submission_clone = submission.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = submission_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "submission",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &submission_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(submission)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/submissions/{id}")]
async fn get_submission_by_id(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    // Get the submission first to check student_id
    let submission = match service.find_one_submission(Some(&id), None).await {
        Ok(sub) => sub,
        Err(err) => return HttpResponse::NotFound().json(err),
    };

    // Students can only view their own submissions
    if matches!(user.role, Some(UserRole::STUDENT)) {
        if let (Some(student_id), Ok(user_oid)) =
            (submission.student_id, parse_object_id_value(&user.id))
        {
            let student_collection = db.collection::<mongodb::bson::Document>("students");
            match student_collection.find_one(doc! { "_id": student_id, "user_id": user_oid })
                .await
            {
                Ok(None) | Err(_) => {
                    return HttpResponse::Forbidden().json(serde_json::json!({
                        "message": "You can only view your own submissions"
                    }));
                }
                _ => {}
            }
        }
    }
    
    // Parents can view their children's submissions
    if matches!(user.role, Some(UserRole::PARENT)) {
        if let Some(student_id) = submission.student_id {
            let parent_service = ParentService::new(&db);
            let student_id_str = student_id.to_hex();
            
            if let Err(e) = require_parent_child_access(&user, &student_id_str, &parent_service).await {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "message": e
                }));
            }
        } else {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": "Cannot verify parent access: student ID not found"
            }));
        }
    }

    match service
        .get_all_submissions_with_relations(None, Some(1), None, Some(doc! { "_id": match IdType::to_object_id(&id) {
            Ok(oid) => oid,
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid ID"}))
        } }))
        .await
    {
        Ok(mut data) => {
            if let Some(submission) = data.data.pop() {
                HttpResponse::Ok().json(submission)
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "message": "Submission not found"
                }))
            }
        }
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[put("/submissions/{id}")]
async fn update_submission(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SubmissionPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssignmentService::new(&db);

    // Students can only update their own submissions before grading
    if matches!(user.role, Some(UserRole::STUDENT)) {
        match service.find_one_submission(Some(&id), None).await {
            Ok(submission) => {
                // Check if already graded
                if matches!(submission.status, crate::domain::assignment::SubmissionStatus::Graded)
                {
                    return HttpResponse::Forbidden().json(serde_json::json!({
                        "message": "Cannot update a graded submission"
                    }));
                }

                // Verify ownership
                if let (Some(student_id), Ok(user_oid)) =
                    (submission.student_id, parse_object_id_value(&user.id))
                {
                    let student_collection = db.collection::<mongodb::bson::Document>("students");
                    match student_collection.find_one(doc! { "_id": student_id, "user_id": user_oid })
                        .await
                    {
                        Ok(None) | Err(_) => {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "message": "You can only update your own submissions"
                            }));
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => return HttpResponse::NotFound().json(err),
        }
    }

    match service.update_submission(&id, &data.into_inner()).await {
        Ok(submission) => {
            let submission_clone = submission.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = submission_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "submission",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &submission_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(submission)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_assignments)
        .service(get_assignment_by_id)
        .service(count_assignments)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_assignment)
                .service(update_assignment)
                .service(delete_assignment)
                .service(submit_assignment)
                .service(get_assignment_submissions)
                .service(grade_submission)
                .service(get_submission_by_id)
                .service(update_submission),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "assignments", blueprint);
}







