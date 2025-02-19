use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::{
    controllers::subject_controller::subject_controller_controller::{
        create_subject, delete_subject_by_id, get_all_subject, get_subject_by_id,
        update_subject_by_id,
    },
    libs::{
        functions::object_id::change_string_into_object_id, schemas::subject_schema::SubjectSchema,
    },
    models::{
        request_error_model::ReqErrModel,
        subject_model::subject_model_model::{SubjectModelNew, SubjectModelPut},
    },
    services::subject_services::create_subject_servicer,
    AppState,
};

pub async fn create_subject_handle(
    state: Data<AppState>,
    subject: Json<SubjectModelNew>,
) -> impl Responder {
    let create = create_subject(state.into_inner(), subject.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn create_subjects_handle(
    state: Data<AppState>,
    subject: Json<SubjectSchema>,
) -> impl Responder {
    let create = create_subject_servicer(state.into_inner(), subject.into_inner()).await;
    match create {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Created().json(r),
    }
}

pub async fn get_all_subject_handle(state: Data<AppState>) -> impl Responder {
    let get = get_all_subject(state.into_inner()).await;
    match get {
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
            message: e.to_string(),
        }),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

pub async fn get_subject_by_id_handle(state: Data<AppState>, id: Path<String>) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match get_subject_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn delete_subject_by_id_handle(
    state: Data<AppState>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match delete_subject_by_id(state.into_inner(), i).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}

pub async fn update_subject_by_id_handle(
    state: Data<AppState>,
    subject: Json<SubjectModelPut>,
    id: Path<String>,
) -> impl Responder {
    match change_string_into_object_id(id.into_inner()) {
        Err(e) => HttpResponse::BadRequest().json(e),
        Ok(i) => match update_subject_by_id(state.into_inner(), i, subject.into_inner()).await {
            Err(e) => HttpResponse::BadRequest().json(ReqErrModel {
                message: e.to_string(),
            }),
            Ok(r) => HttpResponse::Ok().json(r),
        },
    }
}
