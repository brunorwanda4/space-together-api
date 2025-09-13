use actix_web::web;

mod auth;
mod students;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    students::init(cfg);
    auth::init(cfg);
}
