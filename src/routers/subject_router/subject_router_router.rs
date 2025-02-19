use actix_web::web::{self, delete, get, post, put};
use std::sync::Arc;

use crate::{
    handlers::subject_handle::subject_handle_handle::{
        create_subject_handle, create_subjects_handle, delete_subject_by_id_handle,
        get_all_subject_handle, get_subject_by_id_handle, update_subject_by_id_handle,
    },
    AppState,
};

pub fn routers_subject(
    cfg: &mut web::ServiceConfig,
    state: Arc<AppState>,
) -> &mut actix_web::web::ServiceConfig {
    cfg.service(
        web::scope("")
            .app_data(web::Data::new(state.clone()))
            .route("", post().to(create_subject_handle))
            .route("/s", post().to(create_subjects_handle))
            .route("", get().to(get_all_subject_handle))
            .route("/{id}", get().to(get_subject_by_id_handle))
            .route("/{id}", delete().to(delete_subject_by_id_handle))
            .route("{id}", put().to(update_subject_by_id_handle)),
    )
}
