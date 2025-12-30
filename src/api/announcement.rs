use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        announcement::{Announcement, AnnouncementPartial},
        auth_user::AuthUserDto,
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{announcement_service::AnnouncementService, event_service::EventService},
    utils::api_utils::build_extra_match,
};

/// ------------------------------------------------------
/// GET /announcements
/// ------------------------------------------------------

#[get("")]
async fn get_all_announcements(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AnnouncementService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
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

/// ------------------------------------------------------
/// GET /announcements/others
/// ------------------------------------------------------

#[get("/others")]
async fn get_all_announcements_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = AnnouncementService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
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

/// ------------------------------------------------------
/// GET /announcements/{id}/others
/// ------------------------------------------------------

#[get("/{id}/others")]
async fn get_announcement_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(&state.db.main_db());

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /announcements/{id}
/// ------------------------------------------------------

#[get("/{id}")]
async fn get_announcement_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(&state.db.main_db());

    match service.find_one(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /announcements/match
/// ------------------------------------------------------

#[get("/match")]
async fn get_announcement_by_match(
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = AnnouncementService::new(&state.db.main_db());
    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /announcements/others/match
/// ------------------------------------------------------

#[get("/others/match")]
async fn get_announcement_by_other_match(
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = AnnouncementService::new(&state.db.main_db());
    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// POST /announcements
/// ------------------------------------------------------

#[post("")]
async fn create_announcement(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Announcement>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let service = AnnouncementService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "announcement",
                        &id.to_hex(),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /announcements/{id}
/// ------------------------------------------------------

#[put("/{id}")]
async fn update_announcement(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<AnnouncementPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(&state.db.main_db());

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "announcement",
                        &id.to_hex(),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// DELETE /announcements/{id   }
/// ------------------------------------------------------

#[delete("/{id}")]
async fn delete_announcement(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let service = AnnouncementService::new(&state.db.main_db());

    let before_delete = service.find_one(Some(&id), None).await.ok();

    match service.delete(&id).await {
        Ok(_) => {
            if let Some(item) = before_delete {
                let cloned = item.clone();
                let state_clone = state.clone();

                actix_rt::spawn(async move {
                    if let Some(id) = cloned.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "announcement",
                            &id.to_hex(),
                            &cloned,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(doc! {
                "message": "Announcement deleted successfully"
            })
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/announcements")
            // GET
            .service(get_all_announcements)
            .service(get_all_announcements_with_relations)
            .service(get_announcement_by_match)
            .service(get_announcement_by_other_match)
            .service(get_announcement_by_id)
            .service(get_announcement_by_id_with_relations)
            // AUTH REQUIRED
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            // MUTATIONS
            .service(create_announcement)
            .service(update_announcement)
            .service(delete_announcement),
    );
}
