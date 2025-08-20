use actix_web::{
    web::{self, scope, ServiceConfig},
    HttpResponse, Responder,
};
use std::sync::Arc;

use super::{
    auth_router::{adapter_router::routers_adapter, user_auth_router::routers_user_auth_router},
    class_router::{
        activities_type_router::routers_activities_type, activity_router::routers_activity,
        class_group_router::routers_class_group, class_room_router::routers_class_room,
        class_room_type_router::routers_class_room_type, class_router_router::routers_class,
        class_type_router::routers_class_type,
    },
    conversation_router::{
        conversation_router_router::routers_conversation, message_router::routers_message,
    },
    database_router::database_status_router::routers_database,
    education_router::education_router_router::routers_education,
    file_router::{file_router_router::routers_file, file_type_route::routers_file_type},
    request_router::{
        request_router_router::routers_request, request_type_router::routers_request_type,
    },
    school_router::{
        school_router_auth::routers_school_auth, school_router_router::routers_school,
        sector_router::routers_sector, trade_router::routers_trade,
    },
    subject_router::{
        subject_router_router::routers_subject, subject_type_router::routers_subject_type,
    },
    user_router::{user_role_router::routers_user_role, user_router_router::routers_user},
};
use crate::{handlers::database_handle::all_end_point_handle::list_all_endpoints, AppState};

/// API version constants for better maintainability
const API_V1: &str = "/api/v0.0.1";
const API_V2: &str = "/api/v0.0.2";

pub fn all_routers(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    // Root endpoint
    cfg.service(scope("/").route("/", web::get().to(root_handler)));

    // API version 1 routes
    configure_api_v1(cfg, state.clone());

    // API version 2 routes (subject-specific for now)
    configure_api_v2(cfg, state);
}

/// Configures API version 1 routes
fn configure_api_v1(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(
        scope(API_V1)
            .route("/", web::get().to(api_v1_handler))
            .route("/endpoints", web::get().to(list_all_endpoints)) // Debug route
            .app_data(web::Data::new(state.clone()))
            .configure(|cfg| configure_auth_routes(cfg, state.clone()))
            .configure(|cfg| configure_user_routes(cfg, state.clone()))
            .configure(|cfg| configure_class_routes(cfg, state.clone()))
            .configure(|cfg| configure_subject_routes(cfg, state.clone()))
            .configure(|cfg| configure_conversation_routes(cfg, state.clone()))
            .configure(|cfg| configure_database_routes(cfg, state.clone()))
            .configure(|cfg| configure_request_routes(cfg, state.clone()))
            .configure(|cfg| configure_education_routes(cfg, state.clone()))
            .configure(|cfg| configure_school_routes(cfg, state.clone()))
            .configure(|cfg| configure_file_routes(cfg, state.clone())),
    );
}

/// Configures API version 2 routes
fn configure_api_v2(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(scope(API_V2).configure(|cfg| configure_subject_routes(cfg, state)));
}

/// Route configuration helpers for better organization
fn configure_auth_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/adapter").configure(|cfg| {
        routers_adapter(cfg, state.clone());
    }))
    .service(web::scope("/auth").configure(|cfg| {
        routers_user_auth_router(cfg, state);
    }));
}

fn configure_user_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/users").configure(|cfg| {
        routers_user_role(cfg, state.clone());
        routers_user(cfg, state);
    }));
}

fn configure_class_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/classes/room").configure(|cfg| {
        routers_class_room_type(cfg, state.clone());
        routers_class_room(cfg, state.clone());
    }))
    .service(scope("/classes/activities").configure(|cfg| {
        routers_activities_type(cfg, state.clone());
        routers_activity(cfg, state.clone());
    }))
    .service(web::scope("/classes").configure(|cfg| {
        routers_class_type(cfg, state.clone());
        routers_class_group(cfg, state.clone());
        routers_class(cfg, state);
    }));
}

fn configure_subject_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/subject").configure(|cfg| {
        routers_subject_type(cfg, state.clone());
        routers_subject(cfg, state);
    }));
}

fn configure_conversation_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/conversations").configure(|cfg| {
        routers_message(cfg, state.clone());
        routers_conversation(cfg, state);
    }));
}

fn configure_database_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/db").configure(|cfg| {
        routers_database(cfg, state);
    }));
}

fn configure_request_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/requests").configure(|cfg| {
        routers_request_type(cfg, state.clone());
        routers_request(cfg, state);
    }));
}

fn configure_education_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/education").configure(|cfg| {
        routers_education(cfg, state);
    }));
}

fn configure_school_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/school").configure(|cfg| {
        routers_sector(cfg, state.clone());
        routers_trade(cfg, state.clone());
        routers_school_auth(cfg, state.clone());
        routers_school(cfg, state);
    }));
}

fn configure_file_routes(cfg: &mut ServiceConfig, state: Arc<AppState>) {
    cfg.service(web::scope("/file").configure(|cfg| {
        routers_file_type(cfg, state.clone());
        routers_file(cfg, state);
    }));
}

/// Handler functions
async fn root_handler() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Space Together API! 🌼")
}

async fn api_v1_handler() -> impl Responder {
    HttpResponse::Ok().body("Space Together API - Version v0.0.1 🌼")
}
