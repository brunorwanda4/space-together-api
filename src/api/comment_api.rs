use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        comment::{Comment, CommentPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{comment_service::CommentService, event_service::EventService},
    utils::{api_utils::build_extra_match, db_utils::get_database},
};

#[get("")]
async fn get_all_comments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

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

#[get("/others")]
async fn get_all_comments_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

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

#[get("/{id}")]
async fn get_comment_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Comment>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

    match service.create(data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(&state_clone, "comment", &id.to_hex(), &cloned)
                        .await;
                }
            });

            HttpResponse::Created().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<CommentPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(&state_clone, "comment", &id.to_hex(), &cloned)
                        .await;
                }
            });

            HttpResponse::Ok().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

    match service.delete(&id).await {
        Ok(comment) => {
            let cloned = comment.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_deleted(&state_clone, "comment", &id.to_hex(), &cloned)
                        .await;
                }
            });

            HttpResponse::Ok().json(comment)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_comments(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_comments(query.filter.clone(), extra_match)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_comments)
        .service(get_all_comments_with_relations)
        .service(count_comments)
        .service(get_comment_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_comment)
                .service(update_comment)
                .service(delete_comment),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "comments", blueprint);
}
