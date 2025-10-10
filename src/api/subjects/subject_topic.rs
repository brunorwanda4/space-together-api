use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::auth_user::AuthUserDto,
    domain::subjects::subject_topic::{SubjectTopic, UpdateSubjectTopic},
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::subject_topic_repo::SubjectTopicRepo,
    services::event_service::EventService,
    services::subjects::subject_topic_service::SubjectTopicService,
};

#[get("")]
async fn get_all_subject_topics(state: web::Data<AppState>) -> impl Responder {
    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    match service.get_all_topics().await {
        Ok(topics) => HttpResponse::Ok().json(topics),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_subject_topic_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    let topic_id = IdType::from_string(path.into_inner());

    match service.get_topic_by_id(&topic_id).await {
        Ok(topic) => HttpResponse::Ok().json(topic),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/learning-outcome/{learning_outcome_id}")]
async fn get_topics_by_learning_outcome(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    let learning_outcome_id = IdType::from_string(path.into_inner());

    match service
        .get_topics_by_learning_outcome_and_parent(&learning_outcome_id, None)
        .await
    {
        Ok(topics) => HttpResponse::Ok().json(topics),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/learning-outcome/{learning_outcome_id}/parent/{parent_topic_id}")]
async fn get_topics_by_learning_outcome_and_parent(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (learning_outcome_str, parent_topic_str) = path.into_inner();
    let learning_outcome_id = IdType::from_string(learning_outcome_str);
    let parent_topic_id = IdType::from_string(parent_topic_str);

    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    match service
        .get_topics_by_learning_outcome_and_parent(&learning_outcome_id, Some(&parent_topic_id))
        .await
    {
        Ok(topics) => HttpResponse::Ok().json(topics),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_subject_topic(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SubjectTopic>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // ✅ admin check
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    match service
        .create_topic_with_events(data.into_inner(), &state)
        .await
    {
        Ok(topic) => {
            // 🔔 Broadcast individual topic event (existing)
            let topic_clone = topic.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = topic_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "subject_topic",
                        &id.to_hex(),
                        &topic_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(topic)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_subject_topic(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSubjectTopic>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let topic_id = IdType::from_string(path.into_inner());
    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    match service
        .update_topic_with_events(&topic_id, data.into_inner(), &state)
        .await
    {
        Ok(topic) => {
            // 🔔 Broadcast individual topic event (existing)
            let topic_clone = topic.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = topic_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "subject_topic",
                        &id.to_hex(),
                        &topic_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(topic)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_subject_topic(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let topic_id = IdType::from_string(path.into_inner());
    let repo = SubjectTopicRepo::new(&state.db.main_db());
    let service = SubjectTopicService::new(&repo);

    // Get topic before deletion for broadcasting
    let topic_before_delete = repo.find_by_id(&topic_id).await.ok().flatten();

    match service.delete_topic_with_events(&topic_id, &state).await {
        Ok(_) => {
            // 🔔 Broadcast individual topic deletion event (existing)
            if let Some(topic) = topic_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = topic.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "subject_topic",
                            &id.to_hex(),
                            &topic,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Subject topic deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subject-topics")
            // Public routes
            .service(get_all_subject_topics) // GET /subject-topics
            .service(get_topics_by_learning_outcome) // /subject-topics/learning-outcome/{learning_outcome_id}
            .service(get_topics_by_learning_outcome_and_parent) //subject-topics/learning-outcome/{learning_outcome_id}/parent/{parent_topic_id}
            .service(get_subject_topic_by_id) // GET /subject-topics/{id}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_subject_topic) // POST /subject-topics
            .service(update_subject_topic) // PUT /subject-topics/{id}
            .service(delete_subject_topic), // DELETE /subject-topics/{id}
    );
}
