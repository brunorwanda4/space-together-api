use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState, domain::{
        auth_user::AuthUserDto,
        class::{Class, UpdateClass},
    }, errors::AppError, helpers::event_helpers::get_school_id_from_request, models::{api_request_model::RequestQuery, id_model::IdType}, services::{class_service::ClassService, event_service::EventService}, utils::{
        api_utils::build_extra_match, db_utils::get_database, object_id::parse_object_id_value,
    }
};

#[get("")]
async fn get_all_classes(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

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

#[get("/others")]
async fn get_all_classes_with_relations(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

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

#[get("/{id}/others")]
async fn get_class_by_id_with_relations(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    match service.find_one_with_relations(Some(&id), None).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/{id}")]
async fn get_class_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    match service.find_one(Some(&id), None).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/match")]
async fn get_class_by_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one(None, extra_match).await {
        Ok(class) => HttpResponse::Ok().json(class),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[get("/others/match")]
async fn get_class_by_other_match(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service.find_one_with_relations(None, extra_match).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

#[post("")]
async fn create_class(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Class>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    let mut class = data.clone();

    if class.creator_id.is_none() {
        let user_id = match parse_object_id_value(&user.id) {
            Ok(id) => id,
            Err(err) => return HttpResponse::BadRequest().json(err),
        };
        class.creator_id = Some(user_id);
    }

    match service.create(class).await {
        Ok(class) => {
            let class_clone = class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[put("/{id}")]
async fn update_class(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateClass>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    match service.update(&id, &data.into_inner()).await {
        Ok(class) => {
            let class_clone = class.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                 get_school_id_from_request(&req),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[delete("/{id}")]
async fn delete_class(
    req: HttpRequest,
    _user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = IdType::from_string(path.into_inner());
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    match service.delete(&id).await {
        Ok(class) => {
            let class_clone = class.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = class_clone.id {
                    EventService::broadcast_deleted(
                        &state_clone,
                        "class",
                        &id.to_hex(),
                        get_school_id_from_request(&req),
                        &class_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(class)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[get("/count")]
async fn count_classes(
    req: HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    let extra_match = match build_extra_match(&query.field, &query.value) {
        Ok(doc) => doc,
        Err(err) => return err,
    };

    match service
        .count_classes(query.filter.clone(), extra_match)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!(count)),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

#[post("/{main_class_id}/subclasses/count/{count}")]
async fn create_many_subclasses_by_class_id(
    user: web::ReqData<AuthUserDto>,
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let db = get_database(&req, &state);
    let service = ClassService::new(&db);

    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin_or_staff(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let (main_class_id_str, count) = path.into_inner();
    let main_class_id = IdType::String(main_class_id_str);

    let user_id = match ObjectId::from_str(&logged_user.id) {
        Ok(i) => i,
        Err(e) => {
            return HttpResponse::BadRequest().json(AppError {
                message: format!("field to change user id into object id: {}", e),
            })
        }
    };

    let count_num = match count.parse::<u8>() {
        Ok(c) if c > 0 => c,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid count value. It must be a positive number."
            }))
        }
    };

    match service
        .create_many_sub_class_by_class_id(&main_class_id, count_num, user_id)
        .await
    {
        Ok(subclasses) => {
            let state_clone = state.clone();
            let subclasses_for_spawn = subclasses.clone();

            // 📡 Broadcast events async
            actix_rt::spawn(async move {
                for subclass in &subclasses_for_spawn {
                    if let Some(id) = subclass.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "class",
                            &id.to_hex(),
                             get_school_id_from_request(&req),
                            subclass,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(subclasses)
        }
        Err(error) => HttpResponse::BadRequest().json(error),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_classes)
        .service(get_all_classes_with_relations)
        .service(get_class_by_match)
        .service(get_class_by_other_match)
        .service(get_class_by_id_with_relations)
        .service(count_classes)
        .service(get_class_by_id)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_class)
                .service(update_class)
                .service(delete_class)
                .service(create_many_subclasses_by_class_id),
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "classes", blueprint);
}
