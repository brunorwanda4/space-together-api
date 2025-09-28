use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::auth_user::AuthUserDto,
    domain::subjects::main_subject::{MainSubject, UpdateMainSubject}, // 👈 you’ll need UpdateMainSubject struct
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::main_subject_repo::MainSubjectRepo,
    services::subjects::main_subject_service::MainSubjectService,
};

#[get("/main-class/{id}")]
async fn get_subjects_by_main_class_id(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_subjects_by_main_class_id(&subject_id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("")]
async fn get_all_subjects(db: web::Data<Database>) -> impl Responder {
    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);

    match service.get_all_subjects().await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_subject_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_subject_by_id(&subject_id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/code/{code}")]
async fn get_subject_by_code(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);
    let code = path.into_inner();

    match service.get_subject_by_code(&code).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_subject(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<MainSubject>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);

    match service.create_subject(data.into_inner()).await {
        Ok(subject) => HttpResponse::Created().json(subject),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateMainSubject>, // 👈 use DTO for updates
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let subject_id = IdType::from_string(path.into_inner());
    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);

    match service.update_subject(&subject_id, data.into_inner()).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_subject(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let subject_id = IdType::from_string(path.into_inner());
    let repo = MainSubjectRepo::new(db.get_ref());
    let service = MainSubjectService::new(&repo);

    match service.delete_subject(&subject_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Main subject deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/main-subjects")
            // Public routes
            .service(get_all_subjects) // GET /main-subjects
            .service(get_subjects_by_main_class_id) // GET /main-subjects/main-class/{id}
            .service(get_subject_by_code) // GET /main-subjects/code/{code}
            .service(get_subject_by_id) // GET /main-subjects/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_subject) // POST /main-subjects
            .service(update_subject) // PUT /main-subjects/{id}
            .service(delete_subject), // DELETE /main-subjects/{id}
    );
}
