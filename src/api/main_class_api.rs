use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        main_class::{MainClass, MainClassPartial},
    },
    models::{api_request_model::RequestQuery, id_model::IdType},
    services::{event_service::EventService, main_class_service::MainClassService},
    utils::api_utils::build_extra_match,
};

/// ------------------------------------------------------
/// GET /main-classes
/// ------------------------------------------------------
#[get("")]
async fn get_all_main_classes(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(&state.db.main_db());

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
/// GET /main-classes/others
/// ------------------------------------------------------
#[get("/others")]
async fn get_all_main_classes_with_relations(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
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
/// GET /main-classes/{id}
/// ------------------------------------------------------
#[get("/{id}")]
async fn get_main_class_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(&state.db.main_db());

    match service.find_one(Some(&id), None).await {
        Ok(main_class) => HttpResponse::Ok().json(main_class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /main-classes/{id}/others
/// ------------------------------------------------------
#[get("/{id}/others")]
async fn get_main_class_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(&state.db.main_db());

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(main_class) => HttpResponse::Ok().json(main_class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// GET /main-classes/match
/// ------------------------------------------------------
#[get("/match")]
async fn get_main_class_by_match(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(main_class) => HttpResponse::Ok().json(main_class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

/// ------------------------------------------------------
/// POST /main-classes
/// ------------------------------------------------------
#[post("")]
async fn create_main_class(
    _user: web::ReqData<AuthUserDto>,
    data: web::Json<MainClass>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(&state.db.main_db());

    match service.create(data.into_inner()).await {
        Ok(main_class) => {
            let clone = main_class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "main_class",
                        &id.to_hex(),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(main_class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// PUT /main-classes/{id}
/// ------------------------------------------------------
#[put("/{id}")]
async fn update_main_class(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<MainClassPartial>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(&state.db.main_db());

    match service.update(&id, &data.into_inner()).await {
        Ok(main_class) => {
            let clone = main_class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "main_class",
                        &id.to_hex(),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(main_class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// DELETE /main-classes/{id}
/// ------------------------------------------------------
#[delete("/{id}")]
async fn delete_main_class(
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let service = MainClassService::new(&state.db.main_db());

    match service.delete(&id).await {
        Ok(main_class) => {
            let clone = main_class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "main_class",
                        &id.to_hex(),
                        &clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(main_class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// GET /main-classes/count
/// ------------------------------------------------------
#[get("/count")]
async fn count_main_classes(
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let service = MainClassService::new(&state.db.main_db());

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.count(query.filter.clone(), extra_match).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

/// ------------------------------------------------------
/// INIT
/// ------------------------------------------------------
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/main-classes")
            .service(get_all_main_classes)
            .service(get_all_main_classes_with_relations)
            .service(get_main_class_by_match)
            .service(count_main_classes)
            .service(get_main_class_by_id)
            .service(get_main_class_by_id_with_relations)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_main_class)
            .service(update_main_class)
            .service(delete_main_class),
    );
}
