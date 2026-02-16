use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    guards::role_guard::check_permission,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::audit_log_service::AuditLogService,
    utils::{api_utils::build_extra_match, db_utils::get_database},
};

#[get("")]
async fn get_all_audit_logs(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

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
async fn get_all_audit_logs_with_relations(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

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

#[get("/{id}")]
async fn get_audit_log_by_id(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(audit_log) => HttpResponse::Ok().json(audit_log),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_audit_log_by_id_with_relations(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_audit_log_by_match(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(audit_log) => HttpResponse::Ok().json(audit_log),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_audit_log_by_other_match(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/count")]
async fn count_audit_logs(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN, SCHOOLSTAFF, or users with audit.view permission
    if let Err(err) = check_permission(&user, "audit.view") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let db = get_database(&req, &state);
    let service = AuditLogService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_audit_logs(query.filter.clone(), extra_match)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_audit_logs)
        .service(get_all_audit_logs_with_relations)
        .service(get_audit_log_by_match)
        .service(count_audit_logs)
        .service(get_audit_log_by_other_match)
        .service(get_audit_log_by_id_with_relations)
        .service(get_audit_log_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "audit-logs", blueprint);
}
