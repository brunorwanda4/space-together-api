use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        class::{
            BulkClassesForSchoolRequest, BulkClassesRequest, BulkUpdateRequest, Class, UpdateClass,
        },
    },
    guards::role_guard,
    models::{api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel},
    repositories::class_repo::ClassRepo,
    services::{class_service::ClassService, event_service::EventService},
};

#[get("")]
async fn get_all_classes(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service
        .get_all_classes(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-school")]
async fn get_all_classes_with_school(state: web::Data<AppState>) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service.get_all_classes_with_school().await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_classes(state: web::Data<AppState>) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service.get_active_classes().await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_class_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.get_class_by_id(&class_id).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-details")]
async fn get_class_by_id_with_details(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let class_id = IdType::from_string(path.into_inner());

    match service.get_class_by_id_with_others(&class_id).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_class_by_username(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let username = path.into_inner();

    match service.get_class_by_username(&username).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}/with-details")]
async fn get_class_by_username_with_details(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let username = path.into_inner();

    match service.get_class_by_username_with_others(&username).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_class_by_code(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let code = path.into_inner();

    match service.get_class_by_code(&code).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}/with-details")]
async fn get_class_by_code_with_details(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let code = path.into_inner();

    match service.get_class_by_code_with_others(&code).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}")]
async fn get_classes_by_school_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let school_id = IdType::from_string(path.into_inner());

    match service.get_classes_by_school_id(&school_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_classes_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.get_classes_by_creator_id(&creator_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/teacher/{teacher_id}")]
async fn get_classes_by_teacher_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let teacher_id = IdType::from_string(path.into_inner());

    match service.get_classes_by_class_teacher_id(&teacher_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/main-class/{main_class_id}")]
async fn get_classes_by_main_class_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let main_class_id = IdType::from_string(path.into_inner());

    match service.get_classes_by_main_class_id(&main_class_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_class(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Class>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // ✅ Only admin, staff, or teachers can create classes
    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service.create_class(data.into_inner()).await {
        Ok(class) => {
            // ✅ Broadcast created class event
            let class_clone = class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(class)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_class(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
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

    let target_class_id_str = path.into_inner();

    // Check if user has permission to update class
    if let Err(err) =
        crate::guards::role_guard::check_class_access(&logged_user, &target_class_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_class_id = IdType::from_string(target_class_id_str);
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service
        .update_class(&target_class_id, data.into_inner())
        .await
    {
        Ok(class) => {
            // 🔔 Broadcast real-time event
            let class_clone = class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(class)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/merged")]
async fn update_class_merged(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
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

    let target_class_id_str = path.into_inner();

    // Check if user has permission to update class
    if let Err(err) =
        crate::guards::role_guard::check_class_access(&logged_user, &target_class_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_class_id = IdType::from_string(target_class_id_str);
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service
        .update_class_merged(&target_class_id, data.into_inner())
        .await
    {
        Ok(class) => {
            // 🔔 Broadcast real-time event
            let class_clone = class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(class)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_class(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let target_class_id_str = path.into_inner();

    // Only admin or class teacher can delete classes
    if let Err(err) =
        crate::guards::role_guard::check_admin_or_class_teacher(&logged_user, &target_class_id_str)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let target_class_id = IdType::from_string(target_class_id_str);
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    // Get class before deletion for broadcasting
    let class_before_delete = repo.find_by_id(&target_class_id).await.ok().flatten();

    match service.delete_class(&target_class_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(class) = class_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = class.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "class",
                            &id.to_hex(),
                            &class,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Class deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-school/{school_id}")]
async fn count_classes_by_school_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let school_id = IdType::from_string(path.into_inner());

    match service.count_classes_by_school_id(&school_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_classes_by_creator_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let creator_id = IdType::from_string(path.into_inner());

    match service.count_classes_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple classes
#[post("/bulk")]
async fn create_many_classes(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkClassesRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service.create_many_classes(data.classes.clone()).await {
        Ok(classes) => {
            let state_clone = state.clone();
            let classes_for_spawn = classes.clone();

            actix_rt::spawn(async move {
                for class in &classes_for_spawn {
                    if let Some(id) = class.id {
                        EventService::broadcast_created(&state_clone, "class", &id.to_hex(), class)
                            .await;
                    }
                }
            });

            HttpResponse::Created().json(classes)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple classes with validation
#[post("/bulk/validation")]
async fn create_many_classes_with_validation(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkClassesRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service
        .create_many_classes_with_validation(data.classes.clone())
        .await
    {
        Ok(classes) => {
            let state_clone = state.clone();
            let classes_for_spawn = classes.clone();

            actix_rt::spawn(async move {
                for class in &classes_for_spawn {
                    if let Some(id) = class.id {
                        EventService::broadcast_created(&state_clone, "class", &id.to_hex(), class)
                            .await;
                    }
                }
            });

            HttpResponse::Created().json(classes)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple classes for a specific school
#[post("/bulk/school/{school_id}")]
async fn create_many_classes_for_school(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<BulkClassesRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let school_id = path.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({ "message": err.to_string() }));
    }

    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);
    let school_id_typed = IdType::from_string(school_id);

    match service
        .create_many_classes_for_school(&school_id_typed, data.classes.clone())
        .await
    {
        Ok(classes) => {
            let state_clone = state.clone();
            let classes_for_spawn = classes.clone();

            actix_rt::spawn(async move {
                for class in &classes_for_spawn {
                    if let Some(id) = class.id {
                        EventService::broadcast_created(&state_clone, "class", &id.to_hex(), class)
                            .await;
                    }
                }
            });

            HttpResponse::Created().json(classes)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update multiple classes
#[put("/bulk")]
async fn update_many_classes(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateRequest>,
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

    let updates: Vec<(IdType, UpdateClass)> = data
        .updates
        .iter()
        .map(|item| {
            let id = IdType::from_string(item.id.clone());
            (id, item.update.clone())
        })
        .collect();

    // ✅ FIX: IdType might not implement Display — use to_hex() instead of to_string()
    for (id, _) in &updates {
        if let Err(err) =
            crate::guards::role_guard::check_class_access(&logged_user, &id.as_string())
        {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": format!("No permission to update class: {}", err)
            }));
        }
    }

    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    match service.update_many_classes(updates).await {
        Ok(classes) => {
            let state_clone = state.clone();
            let classes_for_spawn = classes.clone();

            actix_rt::spawn(async move {
                for class in &classes_for_spawn {
                    if let Some(id) = class.id {
                        EventService::broadcast_updated(&state_clone, "class", &id.to_hex(), class)
                            .await;
                    }
                }
            });

            HttpResponse::Ok().json(classes)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare classes for bulk creation (helper endpoint)
#[post("/bulk/prepare")]
async fn prepare_classes_for_bulk_creation(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkClassesForSchoolRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Only admin, staff, or teachers can prepare classes
    if let Err(err) = crate::guards::role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = ClassRepo::new(&state.db.main_db());
    let service = ClassService::new(&repo);

    let school_id_typed = IdType::from_string(data.school_id.clone());
    let school_id_obj = match school_id_typed.to_object_id() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid school ID".to_string(),
            })
        }
    };

    let creator_id_obj = match ObjectId::from_str(&logged_user.id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid user ID".to_string(),
            })
        }
    };

    // Use the helper method to prepare classes
    match service.prepare_classes_for_bulk_creation(
        data.classes.clone(),
        Some(school_id_obj),
        Some(creator_id_obj),
    ) {
        Ok(prepared_classes) => HttpResponse::Ok().json(prepared_classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/classes")
            // Public routes (read-only)
            .service(get_all_classes) // GET /classes - Get all classes
            .service(get_all_classes_with_school) // GET /classes/with-school - Get all classes with school information
            .service(get_active_classes) // GET /classes/active - Get all active classes
            .service(get_class_by_id_with_details) // GET /classes/{id}/with-details - Get class by ID with full details
            .service(get_class_by_username) // GET /classes/username/{username} - Get class by username
            .service(get_class_by_username_with_details) // GET /classes/username/{username}/with-details - Get class by username with full details
            .service(get_class_by_code) // GET /classes/code/{code} - Get class by code
            .service(get_class_by_code_with_details) // GET /classes/code/{code}/with-details - Get class by code with full details
            .service(get_classes_by_school_id) // GET /classes/school/{school_id} - Get classes by school ID
            .service(get_classes_by_creator_id) // GET /classes/creator/{creator_id} - Get classes by creator ID
            .service(get_classes_by_teacher_id) // GET /classes/teacher/{teacher_id} - Get classes by teacher ID
            .service(get_classes_by_main_class_id) // GET /classes/main-class/{main_class_id} - Get classes by main class ID
            .service(count_classes_by_school_id) // GET /classes/stats/count-by-school/{school_id} - Count classes by school ID
            .service(count_classes_by_creator_id) // GET /classes/stats/count-by-creator/{creator_id} - Count classes by creator ID
            .service(get_class_by_id) // GET /classes/{id} - Get class by ID
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_class) // POST /classes - Create new class (Admin/Staff/Teacher only)
            .service(update_class) // PUT /classes/{id} - Update class (Admin/ClassTeacher only)
            .service(update_class_merged) // PUT /classes/{id}/merged - Update class with merge (Admin/ClassTeacher only)
            .service(delete_class) // DELETE /classes/{id} - Delete class (Admin/ClassTeacher only)
            // Bulk operations (protected)
            .service(create_many_classes) // POST /classes/bulk - Create multiple classes
            .service(create_many_classes_with_validation) // POST /classes/bulk/validation - Create multiple classes with validation
            .service(create_many_classes_for_school) // POST /classes/bulk/school/{school_id} - Create multiple classes for school
            .service(update_many_classes) // PUT /classes/bulk - Update multiple classes
            .service(prepare_classes_for_bulk_creation), // POST /classes/bulk/prepare - Prepare classes for bulk creation
    );
}
