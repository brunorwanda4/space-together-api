use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        school_staff::{SchoolStaff, SchoolStaffPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, school_staff_service::SchoolStaffService},
    utils::{
        api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value,
    },
};

/// ------------------------------------------------------
/// GET /school-staff
/// ------------------------------------------------------
#[get("")]
async fn get_all_school_staff(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

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
/// GET /school-staff/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_school_staff_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /school-staff/match
/// ------------------------------------------------------
#[get("/match")]
async fn get_school_staff_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// POST /school-staff
/// ------------------------------------------------------
#[post("")]
async fn create_school_staff(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SchoolStaff>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

    let mut staff = data.clone();

    if staff.creator_id.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        staff.creator_id = Some(user_id);
    }

    match service.create(staff, Some(&state)).await {
        Ok(staff) => {
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(staff)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /school-staff/{id}
/// ------------------------------------------------------
#[put("/{id}")]
async fn update_school_staff(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<SchoolStaffPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(staff) => {
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(staff)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// DELETE /school-staff/{id}
/// ------------------------------------------------------
#[delete("/{id}")]
async fn delete_school_staff(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

    match service.delete(&id).await {
        Ok(staff) => {
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(staff)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /school-staff/count
/// ------------------------------------------------------
#[get("/count")]
async fn count_school_staff(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = SchoolStaffService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count_staff(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// ROUTE BLUEPRINT
/// ------------------------------------------------------
fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_school_staff)
        .service(get_school_staff_by_match)
        .service(count_school_staff)
        .service(get_school_staff_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_school_staff)
                .service(update_school_staff)
                .service(delete_school_staff),
        );
}

/// ------------------------------------------------------
/// INIT
/// ------------------------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "school-staff", blueprint);
}
