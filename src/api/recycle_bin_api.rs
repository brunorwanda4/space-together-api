use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    guards::role_guard::check_admin,
    services::recycle_bin_service::RecycleBinService,
    utils::{db_utils::get_database, object_id::parse_object_id_value},
};

#[derive(Debug, Deserialize)]
struct RecycleBinQuery {
    entity_type: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<i64>,
    skip: Option<i64>,
}

#[get("")]
async fn get_recycle_bin(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<RecycleBinQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN can access recycle bin
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
    let service = RecycleBinService::new(db);

    // Parse dates
    let start_date = query
        .start_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let end_date = query
        .end_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    match service
        .get_deleted_entities(
            query.entity_type.clone(),
            start_date,
            end_date,
            school_id,
            query.limit,
            query.skip,
        )
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[derive(Debug, Deserialize)]
struct RestoreRequest {
    entity_type: String,
    entity_id: String,
}

#[post("/restore")]
async fn restore_entity(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<RestoreRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN can restore entities
    if let Err(err_msg) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err_msg.to_string()
        }));
    }

    let db = get_database(&req, &state);
    let service = RecycleBinService::new(db);

    match service
        .restore_entity(&data.entity_type, &data.entity_id, &user, &state)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Entity restored successfully"
        })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/permanent")]
async fn permanently_delete_entity(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<RestoreRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Only ADMIN can permanently delete entities
    if let Err(err_msg) = check_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err_msg.to_string()
        }));
    }

    let db = get_database(&req, &state);
    let service = RecycleBinService::new(db);

    match service
        .permanently_delete(&data.entity_type, &data.entity_id, &user, &state)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Entity permanently deleted"
        })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(get_recycle_bin)
            .service(restore_entity)
            .service(permanently_delete_entity),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "recycle-bin", blueprint);
}
