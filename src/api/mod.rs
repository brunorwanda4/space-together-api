use actix_web::web;

mod auth;
mod class;
mod database_status;
mod events;
mod main_class;
mod school;
mod school_collections;
mod sector;
mod students;
mod subject;
mod subjects;
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
    events::init(cfg);
    subjects::main_subject::init(cfg);
    subjects::subject_topic::init(cfg);
    subjects::learning_outcome::init(cfg);
    subjects::subject_progress_configs::init(cfg);
    subjects::subject_learning_material::init(cfg);
    subjects::subject_grading_schemes::init(cfg);
    school::init(cfg);
    class::init(cfg);
    subject::init(cfg);
    school_collections::school_class::init(cfg);
    school_collections::school_subject::init(cfg);
}
