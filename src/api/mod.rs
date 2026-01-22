use actix_web::web;

mod announcement_api;
mod auth_api;
mod class_api;
mod class_subject;
mod class_timetable;
mod comment_api;
mod database_status;
mod education_year;
mod events;
mod join_school_request_api;
mod like_api;
mod main_class_api;
mod school_api;
mod school_collections;
mod school_staff_api;
mod sector_api;
mod students_api;
mod teachers;
mod template_subject;
mod trade_api;
mod users;
mod welcome;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    welcome::init(cfg);

    database_status::init(cfg);
    sector_api::init(cfg);
    auth_api::init(cfg);
    users::init(cfg);
    trade_api::init(cfg);
    main_class_api::init(cfg);
    class_api::init(cfg);
    events::init(cfg);
    school_api::init(cfg);
    students_api::init(cfg);
    teachers::init(cfg);
    school_staff_api::init(cfg);
    join_school_request_api::init(cfg);
    school_collections::school_class_timetable::init(cfg);
    school_collections::school_timetable::init(cfg);
    template_subject::init(cfg);
    class_subject::init(cfg);
    class_timetable::init(cfg);
    education_year::init(cfg);
    announcement_api::init(cfg);
    comment_api::init(cfg);
    like_api::init(cfg);
}
