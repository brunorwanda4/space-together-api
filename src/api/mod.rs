use actix_web::web;

mod auth;
mod class;
mod class_subject;
mod database_status;
mod events;
mod join_school_request;
mod main_class;
mod school;
mod school_collections;
mod sector;
mod students;
mod subject;
mod subjects;
mod teachers;
mod template_subject;
mod trade;
mod users;
mod welcome;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    welcome::init(cfg);

    database_status::init(cfg);
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
    students::init(cfg);
    teachers::init(cfg);
    join_school_request::init(cfg);
    school_collections::school_class::init(cfg);
    school_collections::school_subject::init(cfg);
    school_collections::school_staff::init(cfg);
    school_collections::school_student::init(cfg);
    school_collections::school_teacher::init(cfg);
    template_subject::init(cfg);
    class_subject::init(cfg);
}
