use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        subjects::subject_learning_material::{
            SubjectLearningMaterial, SubjectLearningMaterialRole, UpdateSubjectLearningMaterial,
        },
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::subject_learning_material_repo::SubjectLearningMaterialRepo,
    services::event_service::EventService,
    services::subjects::subject_learning_material_service::SubjectLearningMaterialService,
};

#[get("/reference/{reference_id}")]
async fn get_by_reference_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    let reference_id = IdType::from_string(path.into_inner());

    match service.get_by_reference_id(&reference_id).await {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/role/{role}/reference/{reference_id}")]
async fn get_by_role_and_reference(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    let (role_str, reference_id_str) = path.into_inner();

    // Parse role from string using serde deserialization from a JSON string
    let role: SubjectLearningMaterialRole = match serde_json::from_str(&format!("\"{}\"", role_str))
    {
        Ok(role) => role,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid role format".to_string(),
            })
        }
    };

    let reference_id = IdType::from_string(reference_id_str);

    match service
        .get_by_role_and_reference(&role, &reference_id)
        .await
    {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/active/role/{role}/reference/{reference_id}")]
async fn get_active_materials(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    let (role_str, reference_id_str) = path.into_inner();

    // Parse role from string
    let role: SubjectLearningMaterialRole = match serde_json::from_str(&format!("\"{}\"", role_str))
    {
        Ok(role) => role,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid role format".to_string(),
            })
        }
    };

    let reference_id = IdType::from_string(reference_id_str);

    match service.get_active(&role, &reference_id).await {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/type/{material_type}/role/{role}/reference/{reference_id}")]
async fn get_by_type_and_reference(
    path: web::Path<(String, String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    let (material_type_str, role_str, reference_id_str) = path.into_inner();

    // Parse material type from string using serde deserialization from a JSON string
    let material_type = match serde_json::from_str(&format!("\"{}\"", material_type_str)) {
        Ok(material_type) => material_type,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid material type format".to_string(),
            })
        }
    };

    // Parse role from string
    let role: SubjectLearningMaterialRole = match serde_json::from_str(&format!("\"{}\"", role_str))
    {
        Ok(role) => role,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid role format".to_string(),
            })
        }
    };

    let reference_id = IdType::from_string(reference_id_str);

    match service
        .get_by_type_and_reference(&material_type, &role, &reference_id)
        .await
    {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_material_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    let material_id = IdType::from_string(path.into_inner());

    match service.get_material_by_id(&material_id).await {
        Ok(material) => HttpResponse::Ok().json(material),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_material(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SubjectLearningMaterial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    match service.create_material(data.into_inner()).await {
        Ok(material) => {
            // 🔔 Broadcast real-time event
            let material_clone = material.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = material_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "subject_learning_material",
                        &id.to_hex(),
                        &material_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(material)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_material(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSubjectLearningMaterial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let material_id = IdType::from_string(path.into_inner());
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    match service
        .update_material(&material_id, data.into_inner())
        .await
    {
        Ok(material) => {
            // 🔔 Broadcast real-time event
            let material_clone = material.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = material_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject_learning_material",
                        &id.to_hex(),
                        &material_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(material)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/status/{is_active}")]
async fn toggle_material_status(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<(String, bool)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let (material_id_str, is_active) = path.into_inner();
    let material_id = IdType::from_string(material_id_str);
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    match service
        .toggle_material_status(&material_id, is_active)
        .await
    {
        Ok(material) => {
            // 🔔 Broadcast real-time event for status change
            let material_clone = material.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = material_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject_learning_material",
                        &id.to_hex(),
                        &material_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(material)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_material(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let material_id = IdType::from_string(path.into_inner());
    let repo = SubjectLearningMaterialRepo::new(&state.db);
    let service = SubjectLearningMaterialService::new(&repo);

    // Get material before deletion for broadcasting
    let material_before_delete = repo.find_by_id(&material_id).await.ok().flatten();

    match service.delete_material(&material_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(material) = material_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = material.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "subject_learning_material",
                            &id.to_hex(),
                            &material,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Learning material deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subject-learning-materials")
            // Public routes
            .service(get_by_reference_id) // GET /subject-learning-materials/reference/{reference_id}
            .service(get_by_role_and_reference) // GET /subject-learning-materials/role/{role}/reference/{reference_id}
            .service(get_active_materials) // GET /subject-learning-materials/active/role/{role}/reference/{reference_id}
            .service(get_by_type_and_reference) // GET /subject-learning-materials/type/{material_type}/role/{role}/reference/{reference_id}
            .service(get_material_by_id) // GET /subject-learning-materials/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_material) // POST /subject-learning-materials
            .service(update_material) // PUT /subject-learning-materials/{id}
            .service(toggle_material_status) // PUT /subject-learning-materials/{id}/status/{is_active}
            .service(delete_material), // DELETE /subject-learning-materials/{id}
    );
}
