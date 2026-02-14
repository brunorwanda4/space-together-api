use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        score::{Score, ScorePartial},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, score_service::ScoreService},
    utils::{api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value},
};

#[get("")]
async fn get_all_scores(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

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

#[get("/{id}")]
async fn get_score_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    match service.find_one(&id).await {
        Ok(score) => HttpResponse::Ok().json(score),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/student/{student_id}/exam/{exam_id}")]
async fn get_student_exam_scores(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (student_id_str, exam_id_str) = path.into_inner();
    
    let student_id = match parse_object_id_value(&student_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let exam_id = match parse_object_id_value(&exam_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    match service.get_student_exam_scores(&student_id, &exam_id).await {
        Ok(scores) => HttpResponse::Ok().json(scores),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/{id}/audit-logs")]
async fn get_score_audit_logs(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let score_id = match parse_object_id_value(&path.into_inner()) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    match service.get_audit_logs(&score_id).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("")]
async fn create_score(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Score>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    let mut score = data.clone();

    if score.entered_by.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        score.entered_by = Some(user_id);
    }

    match service.create(score).await {
        Ok(score) => {
            let score_clone = score.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = score_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "score",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &score_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(score)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/bulk")]
async fn create_bulk_scores(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Vec<Score>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    let user_id = match parse_object_id_value(&user.id) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let mut scores = data.into_inner();
    for score in &mut scores {
        if score.entered_by.is_none() {
            score.entered_by = Some(user_id);
        }
    }

    match service.create_many(scores).await {
        Ok(created_scores) => HttpResponse::Created().json(created_scores),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_score(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ScorePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    let user_id = match parse_object_id_value(&user.id) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let update_data = data.into_inner();
    let change_reason = update_data.remarks.clone().flatten();

    match service.update(&id, &update_data, &user_id, change_reason).await {
        Ok(score) => {
            let score_clone = score.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = score_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "score",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &score_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(score)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_score(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ScoreService::new(&db);

    match service.delete(&id).await {
        Ok(score) => {
            let score_clone = score.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = score_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "score",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &score_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(score)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_scores)
        .service(get_score_by_id)
        .service(get_student_exam_scores)
        .service(get_score_audit_logs)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_score)
                .service(create_bulk_scores)
                .service(update_score)
                .service(delete_score),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "scores", blueprint);
}
