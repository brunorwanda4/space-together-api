use actix_web::web;

mod announcement;
mod auth;
mod class;
mod class_subject;
mod class_timetable;
mod database_status;
mod education_year;
mod events;
mod join_school_request;
mod main_class_api;
mod school;
mod school_collections;
mod school_staff;
mod sector_api;
mod students_api;
mod subject;
mod subjects;
mod teachers;
mod template_subject;
mod trade_api;
mod users;
mod welcome;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    welcome::init(cfg);

    database_status::init(cfg);
    sector_api::init(cfg);
    auth::init(cfg);
    users::init(cfg);
    trade_api::init(cfg);
    // main_class::init(cfg);
    main_class_api::init(cfg);
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
    students_api::init(cfg);
    teachers::init(cfg);
    school_staff::init(cfg);
    join_school_request::init(cfg);
    school_collections::school_class::init(cfg);
    school_collections::school_subject::init(cfg);
    school_collections::school_class_timetable::init(cfg);
    school_collections::school_timetable::init(cfg);
    template_subject::init(cfg);
    class_subject::init(cfg);
    class_timetable::init(cfg);
    education_year::init(cfg);
    announcement::init(cfg);
}
