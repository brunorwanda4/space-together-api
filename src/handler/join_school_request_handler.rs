use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::state::AppState,
    controller::join_school_request_controller::JoinSchoolRequestController,
    domain::join_school_request::{
        BulkCreateJoinSchoolRequest, BulkRespondRequest, CreateJoinSchoolRequest, JoinRequestQuery,
        RespondToJoinRequest, UpdateRequestExpiration,
    },
    errors::AppError,
    models::id_model::IdType,
};

/// Create a join school request
pub async fn create_join_request_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    create_request: web::Json<CreateJoinSchoolRequest>,
    // In real implementation, you'd get sent_by from auth middleware
) -> Result<HttpResponse, AppError> {
    let sent_by = ObjectId::new(); // This should come from authenticated user
    let result = controller
        .create_join_request(create_request.into_inner(), sent_by)
        .await?;
    Ok(HttpResponse::Created().json(result))
}

/// Bulk create join requests
pub async fn bulk_create_join_requests_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    bulk_request: web::Json<BulkCreateJoinSchoolRequest>,
) -> Result<HttpResponse, AppError> {
    let sent_by = ObjectId::new(); // This should come from authenticated user
    let result = controller
        .bulk_create_join_requests(bulk_request.into_inner(), sent_by)
        .await?;
    Ok(HttpResponse::Created().json(result))
}

/// Accept a join request
pub async fn accept_join_request_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    respond_request: web::Json<RespondToJoinRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let accepted_by = Some(ObjectId::new()); // This should come from authenticated user
    let result = controller
        .accept_join_request(respond_request.into_inner(), accepted_by, state)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Reject a join request
pub async fn reject_join_request_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    respond_request: web::Json<RespondToJoinRequest>,
) -> Result<HttpResponse, AppError> {
    let rejected_by = Some(ObjectId::new()); // This should come from authenticated user
    let result = controller
        .reject_join_request(respond_request.into_inner(), rejected_by)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Cancel a join request
pub async fn cancel_join_request_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    respond_request: web::Json<RespondToJoinRequest>,
) -> Result<HttpResponse, AppError> {
    let cancelled_by = Some(ObjectId::new()); // This should come from authenticated user
    let result = controller
        .cancel_join_request(respond_request.into_inner(), cancelled_by)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Get join requests with filtering
pub async fn get_join_requests_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    query: web::Query<JoinRequestQuery>,
) -> Result<HttpResponse, AppError> {
    let requests = controller.get_join_requests(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(requests))
}

/// Get join requests with relations
pub async fn get_join_requests_with_relations_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    query: web::Query<JoinRequestQuery>,
) -> Result<HttpResponse, AppError> {
    let requests = controller
        .get_join_requests_with_relations(query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(requests))
}

/// Get pending requests for a school
pub async fn get_pending_requests_for_school_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    school_id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let school_id = IdType::String(school_id.into_inner());
    let requests = controller
        .get_pending_requests_for_school(&school_id)
        .await?;
    Ok(HttpResponse::Ok().json(requests))
}

/// Update request expiration
pub async fn update_request_expiration_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    update_expiration: web::Json<UpdateRequestExpiration>,
) -> Result<HttpResponse, AppError> {
    let result = controller
        .update_request_expiration(update_expiration.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Bulk respond to requests
pub async fn bulk_respond_to_requests_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    bulk_respond: web::Json<BulkRespondRequest>,
) -> Result<HttpResponse, AppError> {
    let result = controller
        .bulk_respond_to_requests(bulk_respond.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Expire old requests (admin/cron endpoint)
pub async fn expire_old_requests_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
) -> Result<HttpResponse, AppError> {
    let expired_count = controller.expire_old_requests().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "expired_count": expired_count,
        "message": "Old requests expired successfully"
    })))
}

/// Cleanup expired requests (admin/cron endpoint)
pub async fn cleanup_expired_requests_handler(
    controller: web::Data<JoinSchoolRequestController<'_>>,
    older_than_days: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let deleted_count = controller
        .cleanup_expired_requests(older_than_days.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "deleted_count": deleted_count,
        "message": "Expired requests cleaned up successfully"
    })))
}
