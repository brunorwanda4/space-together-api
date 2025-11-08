use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    controller::class_controller::ClassController,
    domain::{
        auth_user::AuthUserDto,
        class::{BulkClassesRequest, BulkUpdateRequest, Class, ClassLevelType, UpdateClass},
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::{
        class_repo::ClassRepo, main_class_repo::MainClassRepo, school_repo::SchoolRepo,
        teacher_repo::TeacherRepo, trade_repo::TradeRepo, user_repo::UserRepo,
    },
    services::{
        class_service::ClassService, event_service::EventService,
        main_class_service::MainClassService, school_service::SchoolService,
        teacher_service::TeacherService, trade_service::TradeService, user_service::UserService,
    },
};

fn create_class_controller(
    state: &web::Data<AppState>,
    claims: &SchoolToken,
) -> ClassController<'static> {
    let school_db = state.db.get_db(&claims.database_name);
    let main_db = state.db.main_db();

    // --- Leak repos so they live for program lifetime ---
    let trade_repo: &'static TradeRepo = Box::leak(Box::new(TradeRepo::new(&main_db)));
    let class_repo = ClassRepo::new(&school_db); // owned, not leaked
    let school_repo: &'static SchoolRepo = Box::leak(Box::new(SchoolRepo::new(&main_db)));
    let user_repo: &'static UserRepo = Box::leak(Box::new(UserRepo::new(&main_db)));
    let main_class_repo: &'static MainClassRepo = Box::leak(Box::new(MainClassRepo::new(&main_db)));
    let teacher_repo: &'static TeacherRepo = Box::leak(Box::new(TeacherRepo::new(&school_db)));

    // --- Leak services (each borrows from the leaked repos) ---
    let trade_service: &'static TradeService = Box::leak(Box::new(TradeService::new(trade_repo)));
    let school_service: &'static SchoolService =
        Box::leak(Box::new(SchoolService::new(school_repo)));
    let user_service: &'static UserService = Box::leak(Box::new(UserService::new(user_repo)));
    let main_class_service: &'static MainClassService = Box::leak(Box::new(MainClassService::new(
        main_class_repo,
        trade_service,
    )));
    let teacher_service: &'static TeacherService =
        Box::leak(Box::new(TeacherService::new(teacher_repo)));

    // --- Build controller ---
    ClassController::new(
        class_repo,
        school_service,
        user_service,
        teacher_service,
        main_class_service,
        trade_service,
    )
}

#[get("")]
async fn get_all_school_classes(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .get_all_classes(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_school_classes(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_active_classes().await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_school_class_by_id(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_by_id(&class_id).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_school_class_by_username(
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

    let username = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_by_username(&username).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_school_class_by_code(
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

    let code = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_by_code(&code).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_school_class(
    req: actix_web::HttpRequest,
    data: web::Json<Class>,
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Set the school_id from the token to ensure consistency
    let mut class_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    class_data.school_id = Some(school_id);

    match service.create_class(class_data).await {
        Ok(class) => {
            // Broadcast created class event
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
async fn update_school_class(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .update_class_merged(&class_id, data.into_inner())
        .await
    {
        Ok(class) => {
            // Broadcast updated class event
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
async fn delete_school_class(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Get class before deletion for broadcasting
    let class_before_delete = repo.find_by_id(&class_id).await.ok().flatten();

    match service.delete_class(&class_id).await {
        Ok(_) => {
            // Broadcast deleted class event
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

#[get("/stats/count")]
async fn count_school_classes(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .count_classes_by_school_id(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

// UPDATED ENDPOINTS USING ClassController

#[get("/with-details")]
async fn get_all_school_classes_with_details(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_all_classes_with_school(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/with-others")]
async fn get_all_school_classes_with_others(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_all_school_classes_with_others(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

#[get("/{id}/with-details")]
async fn get_school_class_by_id_with_details(
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
    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_class_by_id_with_others(&class_id)
        .await
    {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/username/{username}/with-details")]
async fn get_school_class_by_username_with_details(
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

    let username = path.into_inner();
    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_class_by_username_with_others(&username)
        .await
    {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/code/{code}/with-details")]
async fn get_school_class_by_code_with_details(
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

    let code = path.into_inner();
    let class_controller = create_class_controller(&state, &claims);

    match class_controller.get_class_by_code_with_others(&code).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_school_classes_by_creator_id(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_classes_by_creator_id(&creator_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/teacher/{teacher_id}")]
async fn get_school_classes_by_teacher_id(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_classes_by_class_teacher_id(&teacher_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/main-class/{main_class_id}")]
async fn get_school_classes_by_main_class_id(
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

    let main_class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_classes_by_main_class_id(&main_class_id).await {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[put("/{id}/merged")]
async fn update_school_class_merged(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .update_class_merged(&class_id, data.into_inner())
        .await
    {
        Ok(class) => {
            // Broadcast updated class event
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

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_school_classes_by_creator_id(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.count_classes_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple classes for school
#[post("/bulk")]
async fn create_many_school_classes(
    req: actix_web::HttpRequest,
    data: web::Json<BulkClassesRequest>,
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Set school_id for all classes
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    let mut classes_with_school = data.classes.clone();
    for class in &mut classes_with_school {
        class.school_id = Some(school_id);
    }

    match service.create_many_classes(classes_with_school).await {
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

/// Bulk update multiple classes for school
#[put("/bulk")]
async fn update_many_school_classes(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateRequest>,
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    let updates: Vec<(IdType, UpdateClass)> = data
        .updates
        .iter()
        .map(|item| {
            let id = IdType::from_string(item.id.clone());
            (id, item.update.clone())
        })
        .collect();

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

#[post("/{class_id}/teachers/{teacher_id}/assign")]
async fn add_or_update_class_teacher(
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    // ✅ Step 1: Check for valid school token
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let (class_id, teacher_id) = path.into_inner();
    let cls_id = IdType::from_string(&class_id);
    let tea_id = IdType::from_string(&teacher_id);

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .add_or_update_class_teacher(&cls_id, &tea_id)
        .await
    {
        Ok(updated_class) => {
            let class_state_clone = state.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &class_state_clone,
                    "class",
                    &class_id.clone(),
                    &serde_json::json!({ "action": "update", "class_id": class_id }),
                )
                .await;
            });

            let teacher_state_clone = state.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &teacher_state_clone,
                    "teacher",
                    &teacher_id,
                    &serde_json::json!({ "action": "update", "teacher_id": teacher_id }),
                )
                .await;
            });

            HttpResponse::Ok().json(serde_json::json!(updated_class))
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "message": e.to_string()
        })),
    }
}

// ===========================
// NEW SUBCLASS ENDPOINTS
// ===========================

/// Add a subclass to a main class
#[post("/{main_class_id}/subclasses")]
async fn add_subclass_to_main_class(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Class>,
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

    let main_class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Set the school_id from the token to ensure consistency
    let mut subclass_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    subclass_data.school_id = Some(school_id);

    match service.add_subclass(&main_class_id, subclass_data).await {
        Ok(subclass) => {
            // Broadcast created subclass event
            let subclass_clone = subclass.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = subclass_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &subclass_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(subclass)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Remove a subclass
#[delete("/subclasses/{subclass_id}")]
async fn remove_subclass(
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

    let subclass_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Get subclass before deletion for broadcasting
    let subclass_before_delete = repo.find_by_id(&subclass_id).await.ok().flatten();

    match service.remove_subclass(&subclass_id).await {
        Ok(_) => {
            // Broadcast deleted subclass event
            if let Some(subclass) = subclass_before_delete {
                let state_clone = state.clone();

                actix_rt::spawn(async move {
                    if let Some(id) = subclass.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "class",
                            &id.to_hex(),
                            &subclass,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Subclass deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get all subclasses of a main class
#[get("/{main_class_id}/subclasses")]
async fn get_subclasses(
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

    let main_class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_subclasses(&main_class_id).await {
        Ok(subclasses) => HttpResponse::Ok().json(subclasses),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

/// Get subclasses with full details
#[get("/{main_class_id}/subclasses/with-details")]
async fn get_subclasses_with_details(
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

    let main_class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_subclasses_with_others(&main_class_id).await {
        Ok(subclasses) => HttpResponse::Ok().json(subclasses),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

/// Get the parent class of a subclass
#[get("/subclasses/{subclass_id}/parent")]
async fn get_parent_class(
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

    let subclass_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_parent_class(&subclass_id).await {
        Ok(Some(parent_class)) => HttpResponse::Ok().json(parent_class),
        Ok(None) => HttpResponse::NotFound().json(ReqErrModel {
            message: "Parent class not found".to_string(),
        }),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

/// Move a subclass to a different main class
#[put("/subclasses/{subclass_id}/move/{new_main_class_id}")]
async fn move_subclass(
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

    let (subclass_id, new_main_class_id) = path.into_inner();
    let subclass_id = IdType::from_string(subclass_id);
    let new_main_class_id = IdType::from_string(new_main_class_id);

    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .move_subclass(&subclass_id, &new_main_class_id)
        .await
    {
        Ok(updated_subclass) => {
            // Broadcast updated subclass event
            let subclass_clone = updated_subclass.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = subclass_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &subclass_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(updated_subclass)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if a class is a main class with subclasses
#[get("/{class_id}/is-main-with-subclasses")]
async fn is_main_class_with_subclasses(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.is_main_class_with_subclasses(&class_id).await {
        Ok(is_main_with_subclasses) => HttpResponse::Ok().json(serde_json::json!({
            "is_main_with_subclasses": is_main_with_subclasses
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get all main classes
#[get("/main-classes")]
async fn get_main_classes(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .get_main_classes(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(main_classes) => HttpResponse::Ok().json(main_classes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Update subclass information
#[put("/subclasses/{subclass_id}")]
async fn update_subclass(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
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

    let subclass_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service
        .update_subclass(&subclass_id, data.into_inner())
        .await
    {
        Ok(subclass) => {
            // Broadcast updated subclass event
            let subclass_clone = subclass.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = subclass_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &subclass_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(subclass)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create many sub-classes by main class ID
/// Example: POST /classes/64f123abc/subclasses?count=3
#[post("/{main_class_id}/subclasses/count/{count}")]
async fn create_many_subclasses_by_class_id(
    user: web::ReqData<AuthUserDto>,
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    // 🔐 Verify school token
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

    // 🧩 Extract main_class_id
    let (main_class_id_str, count) = path.into_inner();
    let main_class_id = IdType::String(main_class_id_str);

    let user_id = match ObjectId::from_str(&logged_user.id) {
        Ok(i) => i,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("field to change user id into object id: {}", e),
            })
        }
    };

    // 🔢 Get count param
    let count_num = match count.parse::<u8>() {
        Ok(c) if c > 0 => c,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid count value. It must be a positive number."
            }))
        }
    };

    // 🧠 Create controller
    let class_controller = create_class_controller(&state, &claims);

    // 🚀 Call your controller function
    match class_controller
        .create_many_sub_class_by_class_id(&main_class_id, count_num, user_id)
        .await
    {
        Ok(subclasses) => {
            let state_clone = state.clone();
            let subclasses_for_spawn = subclasses.clone();

            // 📡 Broadcast events async
            actix_rt::spawn(async move {
                for subclass in &subclasses_for_spawn {
                    if let Some(id) = subclass.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "class",
                            &id.to_hex(),
                            subclass,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subclasses)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create a main class
#[post("/main-classes")]
async fn create_main_class(
    req: actix_web::HttpRequest,
    data: web::Json<Class>,
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    // Set the school_id from the token to ensure consistency
    let mut main_class_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    main_class_data.school_id = Some(school_id);

    match service.create_main_class(main_class_data).await {
        Ok(main_class) => {
            // Broadcast created main class event
            let main_class_clone = main_class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = main_class_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        &main_class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(main_class)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if a class can be deleted
#[get("/{class_id}/can-delete")]
async fn can_delete_class(
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
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.can_delete_class(&class_id).await {
        Ok(can_delete) => HttpResponse::Ok().json(serde_json::json!({
            "can_delete": can_delete
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get class hierarchy (main class with all its subclasses)
#[get("/{main_class_id}/hierarchy")]
async fn get_class_hierarchy(
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

    let main_class_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = ClassRepo::new(&school_db);
    let service = ClassService::new(&repo);

    match service.get_class_hierarchy(&main_class_id).await {
        Ok((main_class, subclasses)) => HttpResponse::Ok().json(serde_json::json!({
            "main_class": main_class,
            "subclasses": subclasses
        })),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

// ===========================
// NEW SUBCLASS HIERARCHY ENDPOINTS
// ===========================

/// Get main classes with their subclasses (hierarchy data)
#[get("/main-classes/with-subclasses")]
async fn get_main_classes_with_subclasses(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_main_classes_with_subclasses(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(hierarchies) => HttpResponse::Ok().json(hierarchies),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get a specific main class with all its subclasses and full details
#[get("/{main_class_id}/hierarchy")]
async fn get_main_class_hierarchy_with_details(
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

    let main_class_id = IdType::from_string(path.into_inner());
    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_main_class_hierarchy_with_details(&main_class_id)
        .await
    {
        Ok(hierarchy) => HttpResponse::Ok().json(hierarchy),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

/// Get only main classes (without subclasses) with full details
#[get("/main-classes/with-details")]
async fn get_main_classes_with_details(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_main_classes_with_details(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(main_classes) => HttpResponse::Ok().json(main_classes),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get only subclasses (without main classes) with full details
#[get("/subclasses/with-details")]
async fn get_all_subclasses_with_details(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_all_subclasses_with_details(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(subclasses) => HttpResponse::Ok().json(subclasses),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get subclasses by parent class ID with full details
#[get("/{parent_class_id}/subclasses/with-details")]
async fn get_subclasses_by_parent_with_details(
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

    let parent_class_id = IdType::from_string(path.into_inner());
    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_subclasses_by_parent_with_details(&parent_class_id)
        .await
    {
        Ok(subclasses) => HttpResponse::Ok().json(subclasses),
        Err(e) => HttpResponse::NotFound().json(ReqErrModel { message: e.message }),
    }
}

/// Get class statistics (count of main classes, subclasses, etc.)
#[get("/stats/detailed")]
async fn get_class_statistics(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller.get_class_statistics().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Search classes by level type (main class or subclass)
#[get("/search/by-level/{level_type}")]
async fn search_classes_by_level_type(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
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

    let level_type_str = path.into_inner();
    let level_type = match level_type_str.to_lowercase().as_str() {
        "mainclass" | "main" => ClassLevelType::MainClass,
        "subclass" | "sub" => ClassLevelType::SubClass,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid level type. Use 'mainclass' or 'subclass'".to_string(),
            })
        }
    };

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .search_classes_by_level_type(level_type, query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get classes that have no subclasses (empty main classes or subclasses)
#[get("/without-subclasses")]
async fn get_classes_without_subclasses(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_classes_without_subclasses(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get main classes with the most subclasses (top N main classes)
#[get("/main-classes/top-by-subclass-count")]
async fn get_main_classes_by_subclass_count(
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

    let class_controller = create_class_controller(&state, &claims);

    match class_controller
        .get_main_classes_by_subclass_count(query.limit)
        .await
    {
        Ok(classes) => HttpResponse::Ok().json(classes),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/classes")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // Public routes (read-only)
            .service(get_all_school_classes) // GET /school/classes - Get all classes in school
            .service(get_all_school_classes_with_details) // GET /school/classes/with-details - Get all classes with details in school
            .service(get_all_school_classes_with_others) // GET /school/classes/with-others - Get all classes with others
            .service(get_active_school_classes) // GET /school/classes/active - Get active classes in school
            .service(get_school_class_by_username) // GET /school/classes/username/{username} - Get class by username in school
            .service(get_school_class_by_username_with_details) // GET /school/classes/username/{username}/with-details - Get class by username with details in school
            .service(get_school_class_by_code) // GET /school/classes/code/{code} - Get class by code in school
            .service(get_school_class_by_code_with_details) // GET /school/classes/code/{code}/with-details - Get class by code with details in school
            .service(get_school_classes_by_creator_id) // GET /school/classes/creator/{creator_id} - Get classes by creator ID in school
            .service(get_school_classes_by_teacher_id) // GET /school/classes/teacher/{teacher_id} - Get classes by teacher ID in school
            .service(get_school_classes_by_main_class_id) // GET /school/classes/main-class/{main_class_id} - Get classes by main class ID in school
            .service(get_school_class_by_id) // GET /school/classes/{id} - Get class by ID in school
            .service(count_school_classes) // GET /school/classes/stats/count - Count classes in school
            .service(count_school_classes_by_creator_id) // GET /school/classes/stats/count-by-creator/{creator_id} - Count classes by creator ID in school
            // New subclass hierarchy and analytics endpoints
            .service(get_main_classes_with_subclasses) // GET /school/classes/main-classes/with-subclasses - Get main classes with their subclasses
            .service(get_main_class_hierarchy_with_details) // GET /school/classes/{main_class_id}/hierarchy - Get specific main class with all subclasses and details
            .service(get_main_classes_with_details) // GET /school/classes/main-classes/with-details - Get only main classes with full details
            .service(get_all_subclasses_with_details) // GET /school/classes/subclasses/with-details - Get only subclasses with full details
            .service(get_subclasses_by_parent_with_details) // GET /school/classes/{parent_class_id}/subclasses/with-details - Get subclasses by parent with details
            .service(get_class_statistics) // GET /school/classes/stats/detailed - Get detailed class statistics
            .service(search_classes_by_level_type) // GET /school/classes/search/by-level/{level_type} - Search classes by level type
            .service(get_classes_without_subclasses) // GET /school/classes/without-subclasses - Get classes without subclasses
            .service(get_main_classes_by_subclass_count) // GET /school/classes/main-classes/top-by-subclass-count - Get main classes sorted by subclass count
            .service(get_school_class_by_id_with_details) // GET /school/classes/{id}/with-details - Get class by ID with details in school
            // =============================================
            // PROTECTED ROUTES (WRITE OPERATIONS)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            // =============================================
            // Protected routes (require school token)
            .service(create_school_class) // POST /school/classes - Create new class in school
            .service(add_or_update_class_teacher) // POST /school/classes/{class_id}/teachers/{teacher_id}/assign - add or update class teacher
            .service(update_school_class) // PUT /school/classes/{id} - Update class in school
            .service(update_school_class_merged) // PUT /school/classes/{id}/merged - Update class with merge in school
            .service(delete_school_class) // DELETE /school/classes/{id} - Delete class in school
            // Bulk operations for school
            .service(create_many_school_classes) // POST /school/classes/bulk - Create multiple classes in school
            .service(update_many_school_classes) // PUT /school/classes/bulk - Update multiple classes in school
            // New subclass management endpoints
            .service(add_subclass_to_main_class) // POST /school/classes/{main_class_id}/subclasses - Add subclass to main class
            .service(remove_subclass) // DELETE /school/classes/subclasses/{subclass_id} - Remove subclass
            .service(get_subclasses) // GET /school/classes/{main_class_id}/subclasses - Get all subclasses of a main class
            .service(get_subclasses_with_details) // GET /school/classes/{main_class_id}/subclasses/with-details - Get subclasses with full details
            .service(get_parent_class) // GET /school/classes/subclasses/{subclass_id}/parent - Get parent class of subclass
            .service(move_subclass) // PUT /school/classes/subclasses/{subclass_id}/move/{new_main_class_id} - Move subclass to different main class
            .service(is_main_class_with_subclasses) // GET /school/classes/{class_id}/is-main-with-subclasses - Check if class is main class with subclasses
            .service(get_main_classes) // GET /school/classes/main-classes - Get all main classes
            .service(update_subclass) // PUT /school/classes/subclasses/{subclass_id} - Update subclass
            .service(create_many_subclasses_by_class_id) // POST /school/classes/{main_class_id}/subclasses/count/{count} - Bulk add multiple subclasses
            .service(create_main_class) // POST /school/classes/main-classes - Create main class
            .service(can_delete_class) // GET /school/classes/{class_id}/can-delete - Check if class can be deleted
            .service(get_class_hierarchy), // GET /school/classes/{main_class_id}/hierarchy - Get class hierarchy
    );
}
