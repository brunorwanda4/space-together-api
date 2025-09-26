use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::auth_user::AuthUserDto,
    domain::subjects::learning_outcome::{LearningOutcome, UpdateLearningOutcome},
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::learning_outcome_repo::LearningOutcomeRepo,
    services::subjects::learning_outcome_service::LearningOutcomeService,
};

#[get("")]
async fn get_all_outcomes(db: web::Data<Database>) -> impl Responder {
    let repo = LearningOutcomeRepo::new(db.get_ref());
    let service = LearningOutcomeService::new(&repo);

    match service.get_all_outcomes().await {
        Ok(outcomes) => HttpResponse::Ok().json(outcomes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_outcome_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = LearningOutcomeRepo::new(db.get_ref());
    let service = LearningOutcomeService::new(&repo);

    let outcome_id = IdType::from_string(path.into_inner());

    match service.get_outcome_by_id(&outcome_id).await {
        Ok(outcome) => HttpResponse::Ok().json(outcome),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_outcome(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<LearningOutcome>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = LearningOutcomeRepo::new(db.get_ref());
    let service = LearningOutcomeService::new(&repo);

    match service.create_outcome(data.into_inner()).await {
        Ok(outcome) => HttpResponse::Created().json(outcome),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_outcome(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateLearningOutcome>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let outcome_id = IdType::from_string(path.into_inner());
    let repo = LearningOutcomeRepo::new(db.get_ref());
    let service = LearningOutcomeService::new(&repo);

    match service.update_outcome(&outcome_id, data.into_inner()).await {
        Ok(outcome) => HttpResponse::Ok().json(outcome),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_outcome(
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

    let outcome_id = IdType::from_string(path.into_inner());
    let repo = LearningOutcomeRepo::new(db.get_ref());
    let service = LearningOutcomeService::new(&repo);

    match service.delete_outcome(&outcome_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Learning outcome deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/learning-outcomes")
            // Public routes
            .service(get_all_outcomes) // GET /learning-outcomes
            .service(get_outcome_by_id) // GET /learning-outcomes/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_outcome) // POST /learning-outcomes
            .service(update_outcome) // PUT /learning-outcomes/{id}
            .service(delete_outcome), // DELETE /learning-outcomes/{id}
    );
}
