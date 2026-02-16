use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        role::{Role, RolePartial},
    },
    guards::role_guard::check_admin,
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, role_service::RoleService},
    utils::{api_utils::build_extra_match, db_utils::get_database},
};

#[get("")]
async fn get_all_roles(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all(query.filter.clone(), query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/others")]
async fn get_all_roles_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}/others")]
async fn get_role_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}")]
async fn get_role_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_role_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_role_by_other_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);
    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_role(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Role>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Only Admin can create roles
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    match service.create_role(data.into_inner()).await {
        Ok(role) => {
            let role_clone = role.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = role_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "role",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &role_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(role)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_role(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<RolePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Only Admin can update roles
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    match service.update_role(&id, &data.into_inner()).await {
        Ok(role) => {
            let role_clone = role.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = role_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "role",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &role_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(role)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_role(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Only Admin can delete roles
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    match service.delete_role(&id).await {
        Ok(role) => {
            let role_clone = role.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = role_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "role",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &role_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(role)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_roles(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count_roles(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/permissions")]
async fn get_default_permissions() -> impl Responder {
    let permissions = RoleService::get_default_permissions();
    HttpResponse::Ok().json(permissions)
}

#[derive(serde::Deserialize)]
struct AssignRoleRequest {
    user_id: String,
    role_id: String,
    school_id: String,
}

#[post("/assign")]
async fn assign_role(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<AssignRoleRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Only Admin can assign roles
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let db = get_database(&req, &state);
    let service = RoleService::new(&db);

    let user_id = IdType::from_string(data.user_id.clone());
    let role_id = IdType::from_string(data.role_id.clone());
    let school_id = IdType::from_string(data.school_id.clone());

    match service
        .assign_role_to_user(&user_id, &role_id, &school_id)
        .await
    {
        Ok(assignment) => HttpResponse::Created().json(assignment),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_roles)
        .service(get_all_roles_with_relations)
        .service(get_role_by_match)
        .service(count_roles)
        .service(get_role_by_other_match)
        .service(get_role_by_id_with_relations)
        .service(get_role_by_id)
        .service(get_default_permissions)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_role)
                .service(update_role)
                .service(delete_role)
                .service(assign_role),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "roles", blueprint);
}
