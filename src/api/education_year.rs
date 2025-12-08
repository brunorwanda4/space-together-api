use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        education_year::{EducationYear, EducationYearPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{education_year_service::EducationYearService, event_service::EventService},
    utils::api_utils::build_extra_match,
};

/// ------------------------------------------------------
/// GET /education-years
/// ------------------------------------------------------
#[get("")]
async fn get_all_education_years(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    let req_result = service
        .get_all(query.filter.clone(), query.limit, query.skip, extra_match)
        .await;

    match req_result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /education-years/others
/// ------------------------------------------------------
#[get("/others")]
async fn get_all_education_years_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = EducationYearService::new(&state.db.main_db());

    match service
        .get_all_with_relations(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// ------------------------------------------------------
/// GET /education-years/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_education_year_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(&state.db.main_db());

    match service.find_one_by_id(&id).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// ------------------------------------------------------
/// GET /education-years/{id}/others
/// ------------------------------------------------------
#[get("/{id}/others")]
async fn get_education_year_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(&state.db.main_db());

    match service.find_one_with_relations(&id).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// ------------------------------------------------------
/// POST /education-years
/// ------------------------------------------------------
#[post("")]
async fn create_education_year(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<EducationYear>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // only admin
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = EducationYearService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(item) => {
            // broadcast event
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "education_year",
                        &id.to_hex(),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(item)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// ------------------------------------------------------
/// PUT /education-years/{id}
/// ------------------------------------------------------
#[put("/{id}")]
async fn update_education_year(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<EducationYearPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // only admin
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(&state.db.main_db());

    match service.update_year(&id, &data.into_inner()).await {
        Ok(item) => {
            // broadcast update
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "education_year",
                        &id.to_hex(),
                        &cloned,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(item)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// ------------------------------------------------------
/// DELETE /education-years/{id}
/// ------------------------------------------------------
#[delete("/{id}")]
async fn delete_education_year(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // only admin
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = EducationYearService::new(&state.db.main_db());

    // fetch before delete (for broadcast)
    let before_delete = service.find_one_by_id(&id).await.ok();

    match service.delete_year(&id).await {
        Ok(_) => {
            // broadcast deleted
            if let Some(item) = before_delete {
                let cloned = item.clone();
                let state_clone = state.clone();

                actix_rt::spawn(async move {
                    if let Some(id) = cloned.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "education_year",
                            &id.to_hex(),
                            &cloned,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Education year deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// ------------------------------------------------------
/// INIT ROUTES
/// ------------------------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/education-years")
            .service(get_all_education_years)
            .service(get_all_education_years_with_relations)
            .service(get_education_year_by_id)
            .service(get_education_year_by_id_with_relations)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_education_year)
            .service(update_education_year)
            .service(delete_education_year),
    );
}
