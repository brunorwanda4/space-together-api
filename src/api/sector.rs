use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    domain::sector::{Sector, UpdateSector},
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::sector_repo::SectorRepo,
    services::event_service::EventService,
    services::sector_service::SectorService,
};

#[get("")]
async fn get_all_sectors(state: web::Data<AppState>) -> impl Responder {
    let repo = SectorRepo::new(&state.db);
    let service = SectorService::new(&repo);

    match service.get_all_sectors().await {
        Ok(sectors) => HttpResponse::Ok().json(sectors),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_sector_by_id(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let repo = SectorRepo::new(&state.db);
    let service = SectorService::new(&repo);

    let sector_id = IdType::from_string(path.into_inner());

    match service.get_sector_by_id(&sector_id).await {
        Ok(sector) => HttpResponse::Ok().json(sector),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_sector_by_username(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SectorRepo::new(&state.db);
    let service = SectorService::new(&repo);

    let username = path.into_inner();

    match service.get_sector_by_username(&username).await {
        Ok(sector) => HttpResponse::Ok().json(sector),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_sector(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Sector>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SectorRepo::new(&state.db);
    let service = SectorService::new(&repo);

    match service.create_sector(data.into_inner()).await {
        Ok(sector) => {
            // 🔔 Broadcast real-time event WITHOUT modifying service
            let sector_clone = sector.clone();
            let state_clone = state.clone();
            actix_web::rt::spawn(async move {
                if let Some(id) = sector_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "sector",
                        &id.to_hex(),
                        &sector_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(sector)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_sector(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSector>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let sector_id = IdType::from_string(path.into_inner());
    let repo = SectorRepo::new(&state.db);
    let service = SectorService::new(&repo);

    match service.update_sector(&sector_id, data.into_inner()).await {
        Ok(sector) => {
            // 🔔 Broadcast real-time event WITHOUT modifying service
            let sector_clone = sector.clone();
            let state_clone = state.clone();
            actix_web::rt::spawn(async move {
                if let Some(id) = sector_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "sector",
                        &id.to_hex(),
                        &sector_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(sector)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_sector(
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

    let sector_id = IdType::from_string(path.into_inner());
    let repo = SectorRepo::new(&state.db);
    let service = SectorService::new(&repo);

    // Get sector before deletion for broadcasting
    let sector_before_delete = repo.find_by_id(&sector_id).await.ok().flatten();

    match service.delete_sector(&sector_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event WITHOUT modifying service
            if let Some(sector) = sector_before_delete {
                let state_clone = state.clone();
                actix_web::rt::spawn(async move {
                    if let Some(id) = sector.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "sector",
                            &id.to_hex(),
                            &sector,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Sector deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sectors")
            // Public routes
            .service(get_all_sectors) // GET /sectors
            .service(get_sector_by_id) // GET /sectors/{id}
            .service(get_sector_by_username)
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_sector) // POST /sectors
            .service(update_sector) // PUT /sectors/{id}
            .service(delete_sector), // DELETE /sectors/{id}
    );
}
