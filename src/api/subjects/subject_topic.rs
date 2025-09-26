use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::auth_user::AuthUserDto,
    domain::subjects::subject_topic::{SubjectTopic, UpdateSubjectTopic},
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::subject_topic_repo::SubjectTopicRepo, // ✅ repository
    services::subjects::subject_topic_service::SubjectTopicService, // ✅ service
};

#[get("")]
async fn get_all_subject_topics(db: web::Data<Database>) -> impl Responder {
    let repo = SubjectTopicRepo::new(db.get_ref());
    let service = SubjectTopicService::new(&repo);

    match service.get_all_topics().await {
        Ok(topics) => HttpResponse::Ok().json(topics),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_subject_topic_by_id(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectTopicRepo::new(db.get_ref());
    let service = SubjectTopicService::new(&repo);

    let topic_id = IdType::from_string(path.into_inner());

    match service.get_topic_by_id(&topic_id).await {
        Ok(topic) => HttpResponse::Ok().json(topic),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_subject_topic(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SubjectTopic>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // ✅ admin check
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectTopicRepo::new(db.get_ref());
    let service = SubjectTopicService::new(&repo);

    match service.create_topic(data.into_inner()).await {
        Ok(topic) => HttpResponse::Created().json(topic),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_subject_topic(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSubjectTopic>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let topic_id = IdType::from_string(path.into_inner());
    let repo = SubjectTopicRepo::new(db.get_ref());
    let service = SubjectTopicService::new(&repo);

    match service.update_topic(&topic_id, data.into_inner()).await {
        Ok(topic) => HttpResponse::Ok().json(topic),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_subject_topic(
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

    let topic_id = IdType::from_string(path.into_inner());
    let repo = SubjectTopicRepo::new(db.get_ref());
    let service = SubjectTopicService::new(&repo);

    match service.delete_topic(&topic_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Subject topic deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subject-topics")
            // Public routes
            .service(get_all_subject_topics) // GET /subject-topics
            .service(get_subject_topic_by_id) // GET /subject-topics/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_subject_topic) // POST /subject-topics
            .service(update_subject_topic) // PUT /subject-topics/{id}
            .service(delete_subject_topic), // DELETE /subject-topics/{id}
    );
}
