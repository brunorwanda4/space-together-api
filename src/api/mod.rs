use actix_web::web;

mod auth;
mod database_status;
mod main_class;
mod sector;
mod students;
mod trade;
mod users;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    database_status::init(cfg);
    students::init(cfg);
    sector::init(cfg);
    auth::init(cfg);
    users::init(cfg);
    trade::init(cfg);
    main_class::init(cfg);
}
