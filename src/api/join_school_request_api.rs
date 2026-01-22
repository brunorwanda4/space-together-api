use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        join_school_request::{CreateJoinSchoolRequest, JoinSchoolByCode},
    },
    guards::role_guard,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{
        event_service::EventService, join_school_request_service::JoinSchoolRequestService,
    },
    utils::api_utils::build_extra_match,
};

/// ------------------------------------------------------
/// GET /join-school-requests
/// ------------------------------------------------------
#[get("")]
async fn get_all_join_requests(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = JoinSchoolRequestService::new(&state.db.main_db());

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

/// ------------------------------------------------------
/// GET /join-school-requests/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_join_request_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service.find_one(Some(&id), None).await {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /join-school-requests/others
/// ------------------------------------------------------
#[get("/others")]
async fn get_join_requests_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = JoinSchoolRequestService::new(&state.db.main_db());

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

/// ------------------------------------------------------
/// POST /join-school-requests/
/// ------------------------------------------------------
#[post("")]
async fn create_join_school(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<CreateJoinSchoolRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let auth_user = user.into_inner();
    let service = JoinSchoolRequestService::new(&state.db.main_db());
    let sent_by = ObjectId::from_str(&auth_user.id).unwrap();
    match service
        .create(data.into_inner(), sent_by, &state.clone().into_inner())
        .await
    {
        Ok(join_school) => {
            let state_clone = state.clone();
            let join_school_clone = join_school.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &state_clone,
                    "join_school_request",
                    "new",
                    None,
                    &join_school_clone,
                )
                .await;
            });
            HttpResponse::Ok().json(join_school)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// POST /join-school-requests/join-by-code
/// ------------------------------------------------------
#[post("/join-by-code")]
async fn join_school_by_code(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<JoinSchoolByCode>,
    state: web::Data<AppState>,
) -> impl Responder {
    let auth_user = user.into_inner();
    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service
        .join_school_by_code(&data.into_inner(), &auth_user.clone(), state.clone())
        .await
    {
        Ok(token) => {
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &state_clone,
                    "join_school_request",
                    "new",
                    None,
                    &serde_json::json!({ "action": "created", "by_user": auth_user.id }),
                )
                .await;
            });
            HttpResponse::Ok().json(token)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /join-school-requests/{id}/accept
/// ------------------------------------------------------
#[put("/{id}/accept")]
async fn accept_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());

    let invited_user_id = match ObjectId::from_str(&logged_user.id) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid user ID"
            }))
        }
    };

    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service
        .accept_request(
            &id,
            invited_user_id,
            Some(invited_user_id),
            state.clone(),
            &logged_user,
        )
        .await
    {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /join-school-requests/{id}/reject
/// ------------------------------------------------------
#[put("/{id}/reject")]
async fn reject_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());

    let responded_by = IdType::from_string(&logged_user.id).to_object_id().ok();

    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service.reject_request(&id, responded_by).await {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /join-school-requests/{id}/cancel
/// ------------------------------------------------------
#[put("/{id}/cancel")]
async fn cancel_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let id = IdType::from_string(path.into_inner());

    let responded_by = IdType::from_string(&logged_user.id).to_object_id().ok();

    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service.cancel_request(&id, responded_by).await {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /join-school-requests/count
/// ------------------------------------------------------
#[get("/count")]
async fn count_join_requests(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = JoinSchoolRequestService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/match")]
async fn get_join_request_by_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = JoinSchoolRequestService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(school) => HttpResponse::Ok().json(school),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}/others")]
async fn get_join_request_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_join_request_by_other_match(
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let service = JoinSchoolRequestService::new(&state.db.main_db());
    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[delete("/admin/cleanup-expired/{older_than_days}")]
async fn cleanup_expired_join_requests(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<i64>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = JoinSchoolRequestService::new(&state.db.main_db());
    match service.cleanup_expired_requests(path.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::BadRequest().json(error),
    }
}

#[get("/my/pending")]
async fn get_my_pending_join_requests(
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let service = JoinSchoolRequestService::new(&state.db.main_db());

    match service.get_my_pending_request(&logged_user.email).await {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(e),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/join-school-requests")
            .service(get_all_join_requests)
            .service(get_join_requests_with_relations)
            .service(get_join_request_by_match)
            .service(get_join_request_by_other_match)
            .service(get_join_request_by_id_with_relations)
            .service(count_join_requests)
            .service(get_join_request_by_id)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(get_my_pending_join_requests)
            .service(create_join_school)
            .service(join_school_by_code)
            .service(accept_join_request)
            .service(reject_join_request)
            .service(cancel_join_request)
            .service(cleanup_expired_join_requests),
    );
}
