use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    guards::role_guard::check_admin,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::backup_service::BackupService,
    utils::{api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value},
};

#[get("")]
async fn get_all_backups(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

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
async fn get_all_backups_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

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
async fn get_backup_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(backup) => HttpResponse::Ok().json(backup),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_backup_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("/manual")]
async fn create_manual_backup(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN can create manual backups
    if let Err(err_msg) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err_msg.to_string()
        }));
    }

    // Get school_id from user context
    let school_id = match user.current_school_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(oid) => oid,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID not found in user context"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

    match service.create_manual_backup(school_id, &user, &state).await {
        Ok(backup) => HttpResponse::Created().json(backup),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{id}/restore")]
async fn restore_backup(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN can restore backups
    if let Err(err_msg) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err_msg.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

    // Verify backup belongs to user's school
    match service.find_one(Some(&id), None).await {
        Ok(backup) => {
            let backup_school_id = backup.school_id.map(|oid| oid.to_hex());
            if backup_school_id != user.current_school_id {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "message": "Cannot restore backup from another school"
                }));
            }
        }
        Err(err) => return HttpResponse::NotFound().json(err),
    }

    match service.restore_backup(&id, &user, &state).await {
        Ok(restore_record) => HttpResponse::Ok().json(restore_record),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_backup(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN can delete backups
    if let Err(err) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

    match service.delete(&id).await {
        Ok(backup) => HttpResponse::Ok().json(backup),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_backups(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = BackupService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_backups(query.filter.clone(), extra_match)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_backups)
        .service(get_all_backups_with_relations)
        .service(get_backup_by_id)
        .service(get_backup_by_id_with_relations)
        .service(count_backups)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_manual_backup)
                .service(restore_backup)
                .service(delete_backup),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "backups", blueprint);
}
