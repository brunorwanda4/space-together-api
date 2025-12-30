use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        studen::{Student, StudentPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::student_servic::StudentService,
    utils::{api_utils::build_extra_match, db_utils::get_database},
};

/// ------------------------------------------------------
/// GET /students
/// ------------------------------------------------------
#[get("")]
async fn get_all_students(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

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
/// GET /students/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_student_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /students/match
/// ------------------------------------------------------
#[get("/match")]
async fn get_student_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// POST /students
/// ------------------------------------------------------
#[post("")]
async fn create_student(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    data: web::Json<Student>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

    match service.create(data.into_inner()).await {
        Ok(student) => HttpResponse::Created().json(student),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /students/{id}
/// ------------------------------------------------------
#[put("/{id}")]
async fn update_student(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<StudentPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(student) => HttpResponse::Ok().json(student),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// DELETE /students/{id}
/// ------------------------------------------------------
#[delete("/{id}")]
async fn delete_student(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

    match service.delete(&id).await {
        Ok(_) => HttpResponse::Ok().json(doc! {
            "message": "Student deleted successfully"
        }),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /students/count
/// ------------------------------------------------------
#[get("/count")]
async fn count_students(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = StudentService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_students(query.filter.clone(), extra_match)
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
    cfg.service(get_all_students)
        .service(get_student_by_id)
        .service(get_student_by_match)
        .service(count_students)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_student)
                .service(update_student)
                .service(delete_student),
        );
}

/// ------------------------------------------------------
/// INIT
/// ------------------------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "students", blueprint);
}
