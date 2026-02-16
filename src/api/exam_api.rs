use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        exam::{Exam, ExamPartial},
    },
    guards::role_guard::check_admin_staff_or_teacher,
    helpers::event_helpers::get_school_id_from_request,
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, exam_service::ExamService},
    utils::{api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value},
};

#[get("")]
async fn get_all_exams(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

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
async fn get_exam_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

    match service.find_one(&id).await {
        Ok(exam) => HttpResponse::Ok().json(exam),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_exam(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Exam>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Admin, Staff, or Teacher can create exams
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }
    
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

    let mut exam = data.clone();

    if exam.created_by.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        exam.created_by = Some(user_id);
    }

    match service.create(exam).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_exam(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ExamPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Admin, Staff, or Teacher can update exams
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }
    
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_exam(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Check permission: Admin, Staff, or Teacher can delete exams
    if let Err(err) = check_admin_staff_or_teacher(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err
        }));
    }
    
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

    match service.delete(&id).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{id}/publish")]
async fn publish_exam(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

    match service.publish(&id).await {
        Ok(exam) => {
            let exam_clone = exam.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = exam_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "exam",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &exam_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(exam)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_exams(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ExamService::new(&db);

    let extra_match = match build_extra_match(&query) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count_exams(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_exams)
        .service(count_exams)
        .service(get_exam_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_exam)
                .service(update_exam)
                .service(delete_exam)
                .service(publish_exam),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "exams", blueprint);
}
