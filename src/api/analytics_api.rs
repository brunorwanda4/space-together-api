use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        analytics::{AttendanceRateQuery, EnrollmentTrendsQuery},
        auth_user::AuthUserDto,
    },
    guards::role_guard::check_permission,
    helpers::event_helpers::get_school_id_from_request,
    models::id_model::IdType,
    services::analytics_service::AnalyticsService,
    utils::db_utils::get_database,
};

// ========== ENROLLMENT TRENDS ==========
#[get("/enrollment-trends")]
async fn get_enrollment_trends(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<EnrollmentTrendsQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: analytics.read.school
    if let Err(err) = check_permission(&user, "analytics.read.school") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => IdType::from_string(id),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = AnalyticsService::new(&db);

    match service
        .get_enrollment_trends(&school_id, query.year)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// ========== ATTENDANCE RATE ==========
#[get("/attendance-rate")]
async fn get_attendance_rate(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    query: web::Query<AttendanceRateQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: analytics.read.school
    if let Err(err) = check_permission(&user, "analytics.read.school") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => IdType::from_string(id),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = AnalyticsService::new(&db);

    match service
        .get_attendance_rate(&school_id, query.from, query.to)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// ========== PASS/FAIL DISTRIBUTION ==========
#[get("/pass-fail-distribution")]
async fn get_pass_fail_distribution(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: analytics.read.school
    if let Err(err) = check_permission(&user, "analytics.read.school") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => IdType::from_string(id),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = AnalyticsService::new(&db);

    match service.get_pass_fail_distribution(&school_id, None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// ========== FEE COLLECTION SUMMARY ==========
#[get("/fee-summary")]
async fn get_fee_summary(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: analytics.read.school
    if let Err(err) = check_permission(&user, "analytics.read.school") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => IdType::from_string(id),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = AnalyticsService::new(&db);

    match service.get_fee_summary(&school_id).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// ========== TEACHER WORKLOAD ==========
#[get("/teacher-workload")]
async fn get_teacher_workload(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: analytics.read.school
    if let Err(err) = check_permission(&user, "analytics.read.school") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }

    let school_id = match get_school_id_from_request(&req) {
        Some(id) => IdType::from_string(id),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "School ID is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = AnalyticsService::new(&db);

    match service.get_teacher_workload(&school_id).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(get_enrollment_trends)
            .service(get_attendance_rate)
            .service(get_pass_fail_distribution)
            .service(get_fee_summary)
            .service(get_teacher_workload),
    );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "analytics", blueprint);
}
