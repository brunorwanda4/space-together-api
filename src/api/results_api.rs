use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    models::api_request_model::RequestQuery,
    services::gpa_calculation_service::GpaCalculationService,
    utils::{db_utils::get_database, object_id::parse_object_id_value},
};

#[post("/calculate/{exam_id}")]
async fn calculate_exam_results(
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

    let education_year_id = match query.education_year_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "education_year_id is required"
            }))
        }
    };

    let school_id = match query.school_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "school_id is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = GpaCalculationService::new(&db);

    match service
        .calculate_class_results(
            &class_id,
            &exam_id,
            &education_year_id,
            &school_id,
            query.term_id.clone(),
        )
        .await
    {
        Ok(results) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Results calculated successfully",
            "count": results.len(),
            "results": results
        })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/student/{student_id}/term/{term_id}")]
async fn get_student_term_results(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (student_id_str, _term_id) = path.into_inner();

    let student_id = match parse_object_id_value(&student_id_str) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let exam_id = match query.exam_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "exam_id is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = GpaCalculationService::new(&db);

    match service.get_student_result(&student_id, &exam_id).await {
        Ok(Some(result)) => HttpResponse::Ok().json(result),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Result not found"
        })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/class/{class_id}/exam/{exam_id}")]
async fn get_class_exam_results(
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

    let education_year_id = match query.education_year_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "education_year_id is required"
            }))
        }
    };

    let school_id = match query.school_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "school_id is required"
            }))
        }
    };

    let db = get_database(&req, &state);
    let service = GpaCalculationService::new(&db);

    match service
        .calculate_class_results(
            &class_id,
            &exam_id,
            &education_year_id,
            &school_id,
            query.term_id.clone(),
        )
        .await
    {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_student_term_results)
        .service(get_class_exam_results)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(calculate_exam_results),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "results", blueprint);
}
