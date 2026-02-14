use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        assessment_category::{AssessmentCategory, AssessmentCategoryPartial},
        auth_user::AuthUserDto,
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{assessment_category_service::AssessmentCategoryService, event_service::EventService},
    utils::{api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value},
};

#[get("")]
async fn get_all_categories(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = AssessmentCategoryService::new(&db);

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
async fn get_category_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssessmentCategoryService::new(&db);

    match service.find_one(&id).await {
        Ok(category) => HttpResponse::Ok().json(category),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/validate/{class_subject_id}")]
async fn validate_weight(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let class_subject_id = match parse_object_id_value(&path.into_inner()) {
        Ok(id) => id,
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let education_year_id = match query.education_year_id.as_ref() {
        Some(id) => match parse_object_id_value(id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        },
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "education_year_id is required"
        })),
    };

    let db = get_database(&req, &state);
    let service = AssessmentCategoryService::new(&db);

    match service.get_total_weight(&class_subject_id, &education_year_id).await {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!({
            "total_weight": total,
            "remaining": 100.0 - total
        })),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("")]
async fn create_category(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<AssessmentCategory>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = AssessmentCategoryService::new(&db);

    let mut category = data.clone();

    if category.created_by.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        category.created_by = Some(user_id);
    }

    match service.create(category).await {
        Ok(category) => {
            let category_clone = category.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = category_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "assessment_category",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &category_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(category)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_category(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<AssessmentCategoryPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssessmentCategoryService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(category) => {
            let category_clone = category.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = category_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "assessment_category",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &category_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(category)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_category(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = AssessmentCategoryService::new(&db);

    match service.delete(&id).await {
        Ok(category) => {
            let category_clone = category.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = category_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "assessment_category",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &category_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(category)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_categories)
        .service(get_category_by_id)
        .service(validate_weight)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_category)
                .service(update_category)
                .service(delete_category),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "assessment-categories", blueprint);
}
