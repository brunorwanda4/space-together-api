use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    models::api_request_model::RequestQuery,
    services::ranking_service::RankingService,
    utils::{db_utils::get_database, object_id::parse_object_id_value},
};

#[post("/calculate/{exam_id}")]
async fn calculate_rankings(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let exam_id = match parse_object_id_value(&path.into_inner()) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let class_id = match query.class_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "class_id is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = RankingService::new(&db);

    match service.calculate_rankings(&class_id, &exam_id).await {
        Ok(rankings) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Rankings calculated successfully",
            "count": rankings.len(),
            "rankings": rankings
        })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/class/{class_id}/exam/{exam_id}")]
async fn get_class_rankings(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (class_id_str, exam_id_str) = path.into_inner();

    let class_id = match parse_object_id_value(&class_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let exam_id = match parse_object_id_value(&exam_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let db = get_database(&req, &state);
    let service = RankingService::new(&db);

    match service.get_class_rankings(&class_id, &exam_id).await {
        Ok(rankings) => HttpResponse::Ok().json(rankings),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/class/{class_id}/exam/{exam_id}/top")]
async fn get_top_students(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (class_id_str, exam_id_str) = path.into_inner();

    let class_id = match parse_object_id_value(&class_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let exam_id = match parse_object_id_value(&exam_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let limit = query.limit.unwrap_or(10);

    let db = get_database(&req, &state);
    let service = RankingService::new(&db);

    match service.get_top_students(&class_id, &exam_id, limit).await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/class/{class_id}/exam/{exam_id}/at-risk")]
async fn get_at_risk_students(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (class_id_str, exam_id_str) = path.into_inner();

    let class_id = match parse_object_id_value(&class_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let exam_id = match parse_object_id_value(&exam_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    // Default threshold is 2.0 GPA
    let threshold = query.gpa_threshold.unwrap_or(2.0);

    let db = get_database(&req, &state);
    let service = RankingService::new(&db);

    match service
        .get_at_risk_students(&class_id, &exam_id, threshold)
        .await
    {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_class_rankings)
        .service(get_top_students)
        .service(get_at_risk_students)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(calculate_rankings),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "rankings", blueprint);
}
