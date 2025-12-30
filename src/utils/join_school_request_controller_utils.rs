use crate::{
    config::state::AppState,
    controller::join_school_request_controller::JoinSchoolRequestController,
    repositories::{
        join_school_request_repo::JoinSchoolRequestRepo, school_repo::SchoolRepo,
        user_repo::UserRepo,
    },
    services::{school_service::SchoolService, user_service::UserService},
};

/// Helper function to create controller instance
pub(crate) fn create_join_school_request_controller(
    state: &AppState,
) -> JoinSchoolRequestController<'static> {
    let db = state.db.main_db();

    // Leak repos so they live for the program lifetime ('static)
    let user_repo: &'static UserRepo = Box::leak(Box::new(UserRepo::new(&db)));
    let school_repo: &'static SchoolRepo = Box::leak(Box::new(SchoolRepo::new(&db)));

    // Leak services too (since they borrow from leaked repos)
    let user_service: &'static UserService = Box::leak(Box::new(UserService::new(user_repo)));
    let school_service: &'static SchoolService =
        Box::leak(Box::new(SchoolService::new(school_repo)));

    let join_request_repo = JoinSchoolRequestRepo::new(&db);

    JoinSchoolRequestController {
        join_request_repo,
        user_service,
        school_service,
    }
}
