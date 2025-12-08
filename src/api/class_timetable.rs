use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        class_timetable::{ClassTimetable, ClassTimetablePartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{class_timetable_service::ClassTimetableService, event_service::EventService},
};

// DTO for generating a blank structure
#[derive(Deserialize)]
pub struct GenerateStructureDto {
    pub class_id: String,
    pub academic_year: String,
    pub start_time: String,
}

/// --------------------------------------
/// GET /class-timetables
/// --------------------------------------
#[get("")]
async fn get_all_timetables(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = ClassTimetableService::new(&state.db.main_db());

    match service
        .get_all(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// --------------------------------------
/// GET /class-timetables/{id}
/// --------------------------------------
#[get("/{id}")]
async fn get_timetable_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = ClassTimetableService::new(&state.db.main_db());

    match service.find_one_by_id(&id).await {
        Ok(timetable) => HttpResponse::Ok().json(timetable),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// GET /class-timetables/class/{class_id}/year/{year}
/// --------------------------------------
#[get("/class/{class_id}/year/{year}")]
async fn get_timetable_by_class_and_year(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (class_id_str, year) = path.into_inner();

    // Convert string to ObjectId
    let class_oid = match IdType::to_object_id(&IdType::from_string(class_id_str)) {
        Ok(oid) => oid,
        Err(e) => return HttpResponse::BadRequest().json(e),
    };

    let service = ClassTimetableService::new(&state.db.main_db());

    match service.find_by_class_and_year(&class_oid, &year).await {
        Ok(timetable) => HttpResponse::Ok().json(timetable),
        Err(message) => HttpResponse::NotFound().json(message),
    }
}

/// --------------------------------------
/// POST /class-timetables/structure-template
/// Helper to get a blank JSON structure for the frontend form
/// --------------------------------------
#[post("/structure-template")]
async fn get_structure_template(data: web::Json<GenerateStructureDto>) -> impl Responder {
    // We only need to convert the ID here
    let class_oid = match IdType::to_object_id(&IdType::from_string(data.class_id.clone())) {
        Ok(oid) => oid,
        Err(e) => return HttpResponse::BadRequest().json(e),
    };

    let structure = ClassTimetableService::generate_default_structure(
        class_oid,
        data.academic_year.clone(),
        &data.start_time,
    );

    HttpResponse::Ok().json(structure)
}

/// --------------------------------------
/// POST /class-timetables
/// --------------------------------------
#[post("")]
async fn create_timetable(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<ClassTimetable>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Check Role (Admin or Director of Studies)
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let service = ClassTimetableService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
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

/// --------------------------------------
/// PUT /class-timetables/{id}
/// --------------------------------------
#[put("/{id}")]
async fn update_timetable(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<ClassTimetablePartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Check Role
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ClassTimetableService::new(&state.db.main_db());

    match service.update_timetable(&id, &data.into_inner()).await {
        Ok(timetable) => {
            // 🔔 Broadcast update event
            let timetable_clone = timetable.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = timetable_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class_timetable",
                        &id.to_hex(),
                        &timetable_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(timetable)
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

/// --------------------------------------
/// DELETE /class-timetables/{id}
/// --------------------------------------
#[delete("/{id}")]
async fn delete_timetable(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Check Role
    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let service = ClassTimetableService::new(&state.db.main_db());

    // Fetch before deletion for broadcast
    let before_delete = service.find_one_by_id(&id).await.ok();

    match service.delete_timetable(&id).await {
        Ok(_) => {
            // 🔔 Broadcast deletion event
            if let Some(timetable) = before_delete {
                let timetable_clone = timetable.clone();
                let state_clone = state.clone();

                actix_rt::spawn(async move {
                    if let Some(id) = timetable_clone.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "class_timetable",
                            &id.to_hex(),
                            &timetable_clone,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Class timetable deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(message),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/class-timetables")
            .service(get_all_timetables) // GET /class-timetables
            .service(get_structure_template) // POST /class-timetables/structure-template (Helper)
            .service(get_timetable_by_class_and_year) // GET /class-timetables/class/{cid}/year/{y}
            .service(get_timetable_by_id) // GET /class-timetables/{id}
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware) // Protect write routes
            .service(create_timetable) // POST /class-timetables
            .service(update_timetable) // PUT /class-timetables/{id}
            .service(delete_timetable), // DELETE /class-timetables/{id}
    );
}
