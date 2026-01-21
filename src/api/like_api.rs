use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        like::{Like, LikePartial},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, like_service::LikeService},
    utils::{api_utils::build_extra_match, db_utils::get_database},
};

#[get("")]
async fn get_all_likes(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

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
async fn get_all_likes_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .get_all_with_relations(query.limit, query.skip, extra_match)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}")]
async fn get_like_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_like_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(like) => HttpResponse::Ok().json(like),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_like_by_other_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);
    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_like_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_like(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Like>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    match service.create(data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "like",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
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

#[put("/{id}")]
async fn update_like(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<LikePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "like",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
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

#[delete("/{id}")]
async fn delete_like(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    match service.delete(&id).await {
        Ok(like) => {
            let cloned = like.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "like",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(like)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_likes(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = LikeService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count_likes(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_likes)
        .service(get_all_likes_with_relations)
        .service(count_likes)
        .service(get_like_by_match)
        .service(get_like_by_other_match)
        .service(get_like_by_id_with_relations)
        .service(get_like_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_like)
                .service(update_like)
                .service(delete_like),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "likes", blueprint);
}
