use crate::{
    error::school::school_request_error::SchoolRequestResult,
    models::school::school_request_model::{SchoolRequestModel, SchoolRequestModelGet},
    AppState,
};
use std::sync::Arc;

pub async fn get_school_request_controller(
    id: String,
    db: Arc<AppState>,
) -> SchoolRequestResult<SchoolRequestModelGet> {
    // Retrieve SchoolRequestModel from the database
    let request: SchoolRequestModel = db
        .db
        .school_request_db
        .get_school_request_by_id(&id)
        .await
        .unwrap();

    // Transform SchoolRequestModel into SchoolRequestModelGet
    let response = SchoolRequestModelGet {
        id: request
            .id
            .map_or("".to_string(), |obj_id| obj_id.to_string()), // ObjectId -> String
        sended_by: request.sended_by.to_string(), // ObjectId -> String
        name: request.name,
        username: request.username,
        email: request.email,
        phone: request.phone,
        description: request.description,
        location: request.location,
        logo: request.logo,
        verified: request.verified,
        school_type: request.school_type,
        education_system: request.education_system,
        school_id: request.school_id.map(|obj_id| obj_id.to_string()), // ObjectId -> Option<String>
        is_private: request.is_private,
        created_at: request.created_at.to_rfc3339_string(), // DateTime -> String
        updated_at: request
            .updated_at
            .map_or("".to_string(), |dt| dt.to_rfc3339_string()), // Option<DateTime> -> Option<String>
    };

    Ok(response)
}
