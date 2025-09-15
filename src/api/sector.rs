use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::auth_user::AuthUserDto,
    domain::sector::{Sector, UpdateSector},
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::sector_repo::SectorRepo,
    services::sector_service::SectorService,
};

#[get("")]
async fn get_all_sectors(db: web::Data<Database>) -> impl Responder {
    let repo = SectorRepo::new(db.get_ref());
    let service = SectorService::new(&repo);

    match service.get_all_sectors().await {
        Ok(sectors) => HttpResponse::Ok().json(sectors),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_sector_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = SectorRepo::new(db.get_ref());
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
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SectorRepo::new(db.get_ref());
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
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SectorRepo::new(db.get_ref());
    let service = SectorService::new(&repo);

    match service.create_sector(data.into_inner()).await {
        Ok(sector) => HttpResponse::Created().json(sector),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_sector(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSector>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let sector_id = IdType::from_string(path.into_inner());
    let repo = SectorRepo::new(db.get_ref());
    let service = SectorService::new(&repo);

    match service.update_sector(&sector_id, data.into_inner()).await {
        Ok(sector) => HttpResponse::Ok().json(sector),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_sector(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let sector_id = IdType::from_string(path.into_inner());
    let repo = SectorRepo::new(db.get_ref());
    let service = SectorService::new(&repo);

    match service.delete_sector(&sector_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Sector deleted successfully"
        })),
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
