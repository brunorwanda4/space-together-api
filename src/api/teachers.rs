use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        teacher::{Teacher, UpdateTeacher},
    },
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, teacher_service::TeacherService},
    utils::{
        api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value,
    },
};

/// ------------------------------------------------------
/// GET /teachers
/// ------------------------------------------------------
#[get("")]
async fn get_all_teachers(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

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
/// GET /teachers/others
/// ------------------------------------------------------

#[get("/others")]
async fn get_all_teachers_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

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
/// GET /teachers/{id}/others
/// ------------------------------------------------------

#[get("/{id}/others")]
async fn get_teacher_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /teachers/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_teacher_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /teachers/match
/// ------------------------------------------------------
#[get("/match")]
async fn get_teacher_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /teachers/others/match
/// ------------------------------------------------------

#[get("/others/match")]
async fn get_teacher_by_other_match(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<RequestQuery>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);
    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// POST /teachers
/// ------------------------------------------------------
#[post("")]
async fn create_teacher(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Teacher>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    let mut teacher = data.clone();

    if teacher.creator_id.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        teacher.creator_id = Some(user_id);
    }

    match service.create(teacher, Some(&state)).await {
        Ok(teacher) => {
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(teacher)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /teachers/{id}
/// ------------------------------------------------------
#[put("/{id}")]
async fn update_teacher(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateTeacher>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(teacher) => {
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// DELETE /teachers/{id}
/// ------------------------------------------------------
#[delete("/{id}")]
async fn delete_teacher(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    match service.delete(&id).await {
        Ok(teacher) => {
            let teacher_clone = teacher.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = teacher_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "teacher",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &teacher_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(teacher)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /teachers/count
/// ------------------------------------------------------
#[get("/count")]
async fn count_teachers(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = TeacherService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_teachers(query.filter.clone(), extra_match)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// ROUTE BLUEPRINT
/// ------------------------------------------------------
fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_teachers)
        .service(get_all_teachers_with_relations)
        .service(get_teacher_by_match)
        .service(get_teacher_by_other_match)
        .service(get_teacher_by_id_with_relations)
        .service(count_teachers)
        .service(get_teacher_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_teacher)
                .service(update_teacher)
                .service(delete_teacher),
        );
}

/// ------------------------------------------------------
/// INIT
/// ------------------------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "teachers", blueprint);
}
