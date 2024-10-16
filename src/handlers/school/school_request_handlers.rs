use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    error::res_req::ResReq,
    errors::MyError,
    models::school::school_request_model::{SchoolRequestModel, SchoolRequestModelNew},
    AppState,
};

pub async fn create_school_request_handlers(
    State(query): State<Arc<AppState>>,
    Json(request): Json<SchoolRequestModelNew>,
) -> impl IntoResponse {
    let find_req_email = query
        .db
        .school_request_db
        .get_school_request_by_email(&request.email.clone())
        .await;

    let find_req_username = query
        .db
        .school_request_db
        .get_school_request_by_username(&request.username.clone())
        .await;

    if find_req_email.is_ok() && find_req_username.is_ok() {
        let err = ResReq {
            success: false,
            message: MyError::SchoolRequestIsReadyExit.to_string(),
        };
        return (StatusCode::OK, Json(err)).into_response();
    }

    if find_req_email.is_ok() {
        let error_response = ResReq {
            success: false,
            message: MyError::SchoolRequestEmailIsReadyExit {
                email: request.email.clone(),
            }
            .to_string(),
        };
        return (StatusCode::OK, Json(error_response)).into_response();
    }

    if find_req_username.is_ok() {
        let error_response = ResReq {
            success: false,
            message: MyError::SchoolRequestUsernameIsReadyExit {
                username: request.username.clone(),
            }
            .to_string(),
        };
        return (StatusCode::OK, Json(error_response)).into_response();
    }

    let create_req = query
        .db
        .school_request_db
        .create_school_request(request)
        .await;

    match create_req {
        Ok(req) => (StatusCode::OK, Json(req)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: MyError::SchoolRequestCanNotCreate.to_string(),
            };

            (StatusCode::OK, Json(error)).into_response()
        }
    }
}

pub async fn get_school_request_handler(
    State(query): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let req = query
        .db
        .school_request_db
        .get_school_request_by_id(&id)
        .await;

    match req {
        Ok(request) => (StatusCode::OK, Json(request)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };

            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
