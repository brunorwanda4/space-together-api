use actix_web::{post, web, HttpMessage, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    models::{id_model::IdType, school_token_model::SchoolToken},
    services::{class_timetable_service::ClassTimetableService, event_service::EventService},
};

/// --------------------------------------
/// POST /school/class-timetables/generate
/// --------------------------------------
#[post("/generate/{class_id}")]
async fn generate_timetable(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let school_claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let service = ClassTimetableService::new(&state.db.main_db());

    let class_id = IdType::from_string(path.into_inner());

    let generate = service
        .generate_timetable(&class_id, &state, &Some(school_claims))
        .await;

    match generate {
        Ok(timetable) => {
            // 🔔 Broadcast creation event
            let timetable_clone = timetable.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = timetable_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class_timetable",
                        &id.to_hex(),
                        &timetable_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(timetable)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/class-timetables")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            .service(generate_timetable),
    );
}
