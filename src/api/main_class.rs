use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    domain::{
        auth_user::AuthUserDto,
        main_class::{MainClass, UpdateMainClass},
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::{main_class_repo::MainClassRepo, trade_repo::TradeRepo},
    services::{
        event_service::EventService, main_class_service::MainClassService,
        trade_service::TradeService,
    },
};

#[get("/trade")]
async fn get_all_main_classes_with_trade(state: web::Data<AppState>) -> impl Responder {
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.get_all_with_trade().await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("")]
async fn get_all_main_classes(state: web::Data<AppState>) -> impl Responder {
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.get_all().await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_main_class_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let id = IdType::from_string(path.into_inner());

    match service.get_by_id(&id).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/{username}")]
async fn get_main_class_by_username(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let username = path.into_inner();

    match service.get_by_username(&username).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/others/{id}")]
async fn get_main_class_by_id_with_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let main_class_id = IdType::from_string(path.into_inner());

    match service.get_by_id_with_others(&main_class_id).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/username/others/{username}")]
async fn get_main_class_by_username_with_others(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    let username = path.into_inner();

    match service.get_by_username_with_others(&username).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_main_class(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<MainClass>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.create_main_class(data.into_inner()).await {
        Ok(item) => {
            // 🔔 Broadcast real-time event
            let item_clone = item.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = item_clone.id {
                    EventService::broadcast_created(
                        &state_clone,
                        "main_class",
                        &id.to_hex(),
                        &item_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Created().json(item)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_main_class(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateMainClass>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let id = IdType::from_string(path.into_inner());
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    match service.update(&id, data.into_inner()).await {
        Ok(item) => {
            // 🔔 Broadcast real-time event
            let item_clone = item.clone();
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                if let Some(id) = item_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "main_class",
                        &id.to_hex(),
                        &item_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(item)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_main_class(
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

    let id = IdType::from_string(path.into_inner());
    let repo = MainClassRepo::new(&state.db.main_db());
    let trade_repo = TradeRepo::new(&state.db.main_db());
    let trade_service = TradeService::new(&trade_repo);
    let service = MainClassService::new(&repo, &trade_service);

    // Get main_class before deletion for broadcasting
    let main_class_before_delete = repo.find_by_id(&id).await.ok().flatten();

    match service.delete(&id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event
            if let Some(main_class) = main_class_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = main_class.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "main_class",
                            &id.to_hex(),
                            &main_class,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "MainClass deleted successfully"
            }))
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/main-classes")
            // Public routes
            .service(get_all_main_classes)
            .service(get_all_main_classes_with_trade)
            .service(get_main_class_by_username)
            .service(get_main_class_by_username_with_others)
            .service(get_main_class_by_id_with_others)
            .service(get_main_class_by_id)
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_main_class)
            .service(update_main_class)
            .service(delete_main_class),
    );
}
