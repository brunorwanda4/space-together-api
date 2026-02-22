use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use futures::StreamExt;
use mongodb::bson;

use crate::{
    config::state::AppState, domain::{
        auth_user::AuthUserDto,
        common_details::UserRole,
        learning_material::{LearningMaterial, LearningMaterialPartial},
    }, guards::role_guard::check_admin_staff_or_teacher, helpers::event_helpers::get_school_id_from_request, models::{api_request_model::RequestQuery, id_model::IdType}, services::{event_service::EventService, learning_material_service::LearningMaterialService}, utils::{api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value}
};

#[get("")]
async fn get_all_materials(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    let mut extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    }.unwrap_or_default();

    if matches!(user.role, Some(UserRole::STUDENT) | Some(UserRole::PARENT)) {
        extra_match.insert("is_published", bson::to_bson(&true).unwrap());
    }

    match service.get_all(query.filter.clone(), query.limit, query.skip, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_materials_with_relations(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    use mongodb::bson;
    let mut extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    }.unwrap_or_default();

    if matches!(user.role, Some(UserRole::STUDENT) | Some(UserRole::PARENT)) {
        extra_match.insert("is_published", bson::to_bson(&true).unwrap());
    }

    match service.get_all_with_relations(query.filter.clone(), query.limit, query.skip, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_material_by_id(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(material) => {
            if !material.is_published && matches!(user.role, Some(UserRole::STUDENT) | Some(UserRole::PARENT)) {
                return HttpResponse::Forbidden().json(serde_json::json!({"message": "Access denied"}));
            }
            HttpResponse::Ok().json(material)
        }
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_material_by_id_with_relations(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => {
            if !data.learning_material.is_published && matches!(user.role, Some(UserRole::STUDENT) | Some(UserRole::PARENT)) {
                return HttpResponse::Forbidden().json(serde_json::json!({"message": "Access denied"}));
            }
            HttpResponse::Ok().json(data)
        }
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_material(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({"message": err}));
    }

    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    let mut material_data: Option<LearningMaterial> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("Multipart error: {}", e)})),
        };

        let field_name = field.name();

        if field_name == Some("data") {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("Read error: {}", e)})),
                };
                bytes.extend_from_slice(&data);
            }
            let json_str = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("UTF-8 error: {}", e)})),
            };
            material_data = match serde_json::from_str(&json_str) {
                Ok(m) => Some(m),
                Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("JSON error: {}", e)})),
            };
        } else if field_name == Some("file") {
            file_name = field.content_disposition().and_then(|cd| cd.get_filename()).map(|s| s.to_string());
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("File read error: {}", e)})),
                };
                bytes.extend_from_slice(&data);
            }
            file_bytes = Some(bytes);
        }
    }

    let mut material = match material_data {
        Some(m) => m,
        None => return HttpResponse::BadRequest().json(serde_json::json!({"message": "Missing material data"})),
    };

    let user_id = match parse_object_id_value(&user.id) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };
    material.uploaded_by = Some(user_id);

    match service.create(material, file_bytes, file_name, &user, &state).await {
        Ok(created) => {
            let created_clone = created.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = created_clone.id {
                    EventService::broadcast_created(&state_clone, "learning_material", &id.to_hex(), get_school_id_from_request(&req), &created_clone).await;
                }
            });
            HttpResponse::Created().json(created)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_material(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({"message": err}));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    let mut update_data: Option<LearningMaterialPartial> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("Multipart error: {}", e)})),
        };

        let field_name = field.name();

        if field_name == Some("data") {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("Read error: {}", e)})),
                };
                bytes.extend_from_slice(&data);
            }
            let json_str = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("UTF-8 error: {}", e)})),
            };
            update_data = match serde_json::from_str(&json_str) {
                Ok(m) => Some(m),
                Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("JSON error: {}", e)})),
            };
        } else if field_name == Some("file") {
            file_name = field.content_disposition().and_then(|cd| cd.get_filename()).map(|s| s.to_string());
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"message": format!("File read error: {}", e)})),
                };
                bytes.extend_from_slice(&data);
            }
            file_bytes = Some(bytes);
        }
    }

    let update = match update_data {
        Some(u) => u,
        None => return HttpResponse::BadRequest().json(serde_json::json!({"message": "Missing update data"})),
    };

    match service.update(&id, &update, file_bytes, file_name, &user, &state).await {
        Ok(updated) => {
            let updated_clone = updated.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = updated_clone.id {
                    EventService::broadcast_updated(&state_clone, "learning_material", &id.to_hex(), get_school_id_from_request(&req), &updated_clone).await;
                }
            });
            HttpResponse::Ok().json(updated)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_material(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({"message": err}));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    match service.delete(&id, &user, &state).await {
        Ok(material) => {
            let material_clone = material.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = material_clone.id {
                    EventService::broadcast_deleted(&state_clone, "learning_material", &id.to_hex(), get_school_id_from_request(&req), &material_clone).await;
                }
            });
            HttpResponse::Ok().json(material)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_materials(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LearningMaterialService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    }.unwrap_or_default();

    match service.count_materials(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(get_all_materials)
            .service(get_all_materials_with_relations)
            .service(get_material_by_id)
            .service(get_material_by_id_with_relations)
            .service(create_material)
            .service(update_material)
            .service(delete_material)
            .service(count_materials),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "learning-materials", blueprint);
}
