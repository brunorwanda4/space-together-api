use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};

use crate::{error::res_req::ResReq, errors::MyError, models::school::school_request_model::SchoolRequestModel, AppState};

pub async fn create_school_request_handlers (
    State(query) : State<Arc<AppState>>,
    Json(request) : Json<SchoolRequestModel>,
) -> impl IntoResponse {
    let create_req =  
        query
        .db
        .school_request_db
        .create_school_request(&request)
        .await;

    match create_req {
        Ok(req) => (StatusCode::OK ,Json(req)).into_response(),
        Err(err) => {
            let error  = ResReq {
                success : false,
                message : MyError::SchoolRequestCanNotCreate.to_string()
            };

            return (StatusCode::BAD_REQUEST,Json(error)).into_response();
        }
    }
}

pub async fn get_school_request_handler(
    State(query) : State<Arc<AppState>>,
    Path(id) : Path<String>
) -> impl IntoResponse  {
    let req = query
        .db
        .school_request_db
        .get_school_request_by_id(&id)
        .await;

    match req {
        Ok(request) => (StatusCode::OK , Json(request)).into_response(),
        Err(err) => {
            let error = ResReq {
                success: false,
                message: err.to_string(),
            };
            
            return (StatusCode::INTERNAL_SERVER_ERROR , Json(error)).into_response();
        }
    }
}