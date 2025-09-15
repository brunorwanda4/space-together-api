use actix_web::web;

mod auth;
mod sector;
mod students;
mod users;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    students::init(cfg);
    sector::init(cfg);
    auth::init(cfg);
    users::init(cfg);
}
