use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{
    config::state::AppState,
    controller::join_school_request_controller::JoinSchoolRequestController,
    domain::{
        auth_user::AuthUserDto,
        join_school_request::{
            BulkCreateJoinSchoolRequest, BulkRespondRequest, CreateJoinSchoolRequest,
            JoinRequestQuery, RespondToJoinRequest, UpdateRequestExpiration,
        },
    },
    guards::role_guard,
    handler::join_school_request_handler::{
        accept_join_request_handler, bulk_create_join_requests_handler,
        bulk_respond_to_requests_handler, cancel_join_request_handler,
        cleanup_expired_requests_handler, create_join_request_handler, expire_old_requests_handler,
        get_join_requests_handler, get_join_requests_with_relations_handler,
        get_pending_requests_for_school_handler, reject_join_request_handler,
        update_request_expiration_handler,
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::{
        join_school_request_repo::JoinSchoolRequestRepo, school_repo::SchoolRepo,
        user_repo::UserRepo,
    },
    services::{
        event_service::EventService, school_service::SchoolService, user_service::UserService,
    },
};

/// Get all join school requests with filtering
#[get("")]
async fn get_all_join_requests(
    query: web::Query<JoinRequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    match get_join_requests_handler(web::Data::new(controller), query).await {
        Ok(response) => response,
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get all join school requests with relations (school, user, sender)
#[get("/with-relations")]
async fn get_all_join_requests_with_relations(
    query: web::Query<JoinRequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    match get_join_requests_with_relations_handler(web::Data::new(controller), query).await {
        Ok(response) => response,
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join request by ID
#[get("/{id}")]
async fn get_join_request_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let request_id = IdType::String(path.into_inner());

    match controller.join_request_repo.find_by_id(&request_id).await {
        Ok(Some(request)) => HttpResponse::Ok().json(request),
        Ok(None) => HttpResponse::NotFound().json(ReqErrModel {
            message: "Join request not found".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join request by ID with relations
#[get("/{id}/with-relations")]
async fn get_join_request_by_id_with_relations(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let request_id = IdType::String(path.into_inner());

    match controller
        .join_request_repo
        .find_with_relations_by_id(&request_id)
        .await
    {
        Ok(Some(request)) => HttpResponse::Ok().json(request),
        Ok(None) => HttpResponse::NotFound().json(ReqErrModel {
            message: "Join request not found".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join requests by email
#[get("/email/{email}")]
async fn get_join_requests_by_email(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let email = path.into_inner();

    match controller
        .join_request_repo
        .find_with_relations_by_email(&email)
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join requests by school ID
#[get("/school/{school_id}")]
async fn get_join_requests_by_school_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let school_id = IdType::String(path.into_inner());

    match controller
        .join_request_repo
        .find_by_school_id(&school_id)
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get pending join requests by school ID
#[get("/school/{school_id}/pending")]
async fn get_pending_join_requests_by_school_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    match get_pending_requests_for_school_handler(web::Data::new(controller), path).await {
        Ok(response) => response,
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join requests by invited user ID
#[get("/user/{user_id}")]
async fn get_join_requests_by_user_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let user_id = IdType::String(path.into_inner());

    match controller
        .join_request_repo
        .find_by_invited_user_id(&user_id)
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join requests by status
#[get("/status/{status}")]
async fn get_join_requests_by_status(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let status_str = path.into_inner();

    let status = match status_str.to_lowercase().as_str() {
        "pending" => crate::domain::join_school_request::JoinStatus::Pending,
        "accepted" => crate::domain::join_school_request::JoinStatus::Accepted,
        "rejected" => crate::domain::join_school_request::JoinStatus::Rejected,
        "expired" => crate::domain::join_school_request::JoinStatus::Expired,
        "cancelled" => crate::domain::join_school_request::JoinStatus::Cancelled,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid status. Must be 'pending', 'accepted', 'rejected', 'expired', or 'cancelled'".to_string(),
            })
        }
    };

    match controller.join_request_repo.find_by_status(status).await {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Create a new join school request
#[post("")]
async fn create_join_request(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<CreateJoinSchoolRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.clone().into_inner();

    // Only admin, staff, or teachers can create join requests
    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    match create_join_request_handler(web::Data::new(controller), data, user).await {
        Ok(response) => {
            // 🔔 Broadcast real-time event for created join request
            // We'll need to extract the created request ID from the response or fetch the latest
            // For now, we'll broadcast a generic event and let clients refetch if needed
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &state_clone,
                    "join_school_request",
                    "new",
                    &serde_json::json!({ "action": "created", "by_user": logged_user.id }),
                )
                .await;
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Create multiple join requests
#[post("/bulk")]
async fn create_bulk_join_requests(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkCreateJoinSchoolRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    match bulk_create_join_requests_handler(web::Data::new(controller), data).await {
        Ok(response) => {
            // 🔔 Broadcast real-time event for bulk creation
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_created(
                    &state_clone,
                    "join_school_request",
                    "bulk",
                    &serde_json::json!({ "action": "bulk_created", "by_user": logged_user.id }),
                )
                .await;
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Accept a join request
#[put("/{id}/accept")]
async fn accept_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Check permissions - user can accept their own requests or admin/staff can accept any
    let request_id = path.into_inner();
    let controller = create_join_school_request_controller(&state);

    // Get the request first to check ownership
    let request = match controller
        .join_request_repo
        .find_by_id(&IdType::String(request_id.clone()))
        .await
    {
        Ok(Some(req)) => req,
        Ok(None) => {
            return HttpResponse::NotFound().json(ReqErrModel {
                message: "Join request not found".to_string(),
            })
        }
        Err(e) => return HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    };

    // Check if user is the invited user or has admin/staff privileges
    let is_invited_user = if let Some(invited_user_id) = request.invited_user_id {
        logged_user.id == invited_user_id.to_string()
    } else {
        // If no invited_user_id is set, check by email
        logged_user.email == request.email
    };

    if !is_invited_user {
        if let Err(_err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": "You can only accept your own join requests".to_string()
            }));
        }
    }

    let respond_request = RespondToJoinRequest {
        request_id: request_id.clone(),
        status: crate::domain::join_school_request::JoinStatus::Accepted,
        responded_by: Some(logged_user.id.to_string()),
        invited_user_id: Some(logged_user.id.to_string()),
        message: None,
    };

    let controller = create_join_school_request_controller(&state);
    match accept_join_request_handler(
        web::Data::new(controller),
        web::Json(respond_request),
        state.clone(),
    )
    .await
    {
        Ok(response) => {
            // 🔔 Broadcast real-time event for updated join request
            let state_clone = state.clone();
            let request_id_clone = request_id.clone();
            actix_rt::spawn(async move {
                let controller = create_join_school_request_controller(&state_clone);
                if let Ok(Some(updated_request)) = controller
                    .join_request_repo
                    .find_by_id(&IdType::String(request_id_clone))
                    .await
                {
                    if let Some(id) = updated_request.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "join_school_request",
                            &id.to_hex(),
                            &updated_request,
                        )
                        .await;
                    }
                }
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Reject a join request
#[put("/{id}/reject")]
async fn reject_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Check permissions - user can reject their own requests or admin/staff can reject any
    let request_id = path.into_inner();
    let controller = create_join_school_request_controller(&state);

    // Get the request first to check ownership
    let request = match controller
        .join_request_repo
        .find_by_id(&IdType::String(request_id.clone()))
        .await
    {
        Ok(Some(req)) => req,
        Ok(None) => {
            return HttpResponse::NotFound().json(ReqErrModel {
                message: "Join request not found".to_string(),
            })
        }
        Err(e) => return HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    };

    // Check if user is the invited user or has admin/staff privileges
    let is_invited_user = if let Some(invited_user_id) = request.invited_user_id {
        logged_user.id == invited_user_id.to_string()
    } else {
        // If no invited_user_id is set, check by email
        logged_user.email == request.email
    };

    if !is_invited_user {
        if let Err(_err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": "You can only reject your own join requests".to_string()
            }));
        }
    }

    let respond_request = RespondToJoinRequest {
        request_id: request_id.clone(),
        status: crate::domain::join_school_request::JoinStatus::Rejected,
        responded_by: Some(logged_user.id.clone()),
        invited_user_id: None,
        message: None,
    };

    let controller = create_join_school_request_controller(&state);
    match reject_join_request_handler(web::Data::new(controller), web::Json(respond_request)).await
    {
        Ok(response) => {
            // 🔔 Broadcast real-time event for updated join request
            let state_clone = state.clone();
            let request_id_clone = request_id.clone();
            actix_rt::spawn(async move {
                let controller = create_join_school_request_controller(&state_clone);
                if let Ok(Some(updated_request)) = controller
                    .join_request_repo
                    .find_by_id(&IdType::String(request_id_clone))
                    .await
                {
                    if let Some(id) = updated_request.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "join_school_request",
                            &id.to_hex(),
                            &updated_request,
                        )
                        .await;
                    }
                }
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Cancel a join request (by sender or admin)
#[put("/{id}/cancel")]
async fn cancel_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    // Check permissions - sender can cancel their own requests or admin/staff can cancel any
    let request_id = path.into_inner();
    let controller = create_join_school_request_controller(&state);

    // Get the request first to check ownership
    let request = match controller
        .join_request_repo
        .find_by_id(&IdType::String(request_id.clone()))
        .await
    {
        Ok(Some(req)) => req,
        Ok(None) => {
            return HttpResponse::NotFound().json(ReqErrModel {
                message: "Join request not found".to_string(),
            })
        }
        Err(e) => return HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    };

    // Check if user is the sender or has admin/staff privileges
    let is_sender = logged_user.id == request.sent_by.to_string();

    if !is_sender {
        if let Err(_err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "message": "You can only cancel join requests you sent".to_string()
            }));
        }
    }

    let respond_request = RespondToJoinRequest {
        request_id: request_id.clone(),
        status: crate::domain::join_school_request::JoinStatus::Cancelled,
        responded_by: Some(logged_user.id.clone()),
        invited_user_id: None,
        message: None,
    };

    let controller = create_join_school_request_controller(&state);
    match cancel_join_request_handler(web::Data::new(controller), web::Json(respond_request)).await
    {
        Ok(response) => {
            // 🔔 Broadcast real-time event for updated join request
            let state_clone = state.clone();
            let request_id_clone = request_id.clone();
            actix_rt::spawn(async move {
                let controller = create_join_school_request_controller(&state_clone);
                if let Ok(Some(updated_request)) = controller
                    .join_request_repo
                    .find_by_id(&IdType::String(request_id_clone))
                    .await
                {
                    if let Some(id) = updated_request.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "join_school_request",
                            &id.to_hex(),
                            &updated_request,
                        )
                        .await;
                    }
                }
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Update join request expiration
#[put("/{id}/expiration")]
async fn update_join_request_expiration(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateRequestExpiration>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    let mut update_data = data.into_inner();
    let request_id = path.into_inner();
    update_data.request_id = request_id.clone();

    match update_request_expiration_handler(web::Data::new(controller), web::Json(update_data))
        .await
    {
        Ok(response) => {
            // 🔔 Broadcast real-time event for updated join request
            let state_clone = state.clone();
            let request_id_clone = request_id.clone();
            actix_rt::spawn(async move {
                let controller = create_join_school_request_controller(&state_clone);
                if let Ok(Some(updated_request)) = controller
                    .join_request_repo
                    .find_by_id(&IdType::String(request_id_clone))
                    .await
                {
                    if let Some(id) = updated_request.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "join_school_request",
                            &id.to_hex(),
                            &updated_request,
                        )
                        .await;
                    }
                }
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Bulk respond to join requests (accept/reject/cancel)
#[put("/bulk/respond")]
async fn bulk_respond_to_join_requests(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<BulkRespondRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    match bulk_respond_to_requests_handler(web::Data::new(controller), data).await {
        Ok(response) => {
            // 🔔 Broadcast real-time events for bulk update
            let state_clone = state.clone();
            actix_rt::spawn(async move {
                EventService::broadcast_updated(
                    &state_clone,
                    "join_school_request",
                    "bulk",
                    &serde_json::json!({ "action": "bulk_updated", "by_user": logged_user.id }),
                )
                .await;
            });
            response
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Delete a join request
#[delete("/{id}")]
async fn delete_join_request(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    let request_id = IdType::String(path.into_inner());

    // Get request before deletion for broadcasting
    let request_before_delete = controller
        .join_request_repo
        .find_by_id(&request_id)
        .await
        .ok()
        .flatten();

    match controller.join_request_repo.delete(&request_id).await {
        Ok(_) => {
            // 🔔 Broadcast real-time event for deleted join request
            if let Some(request) = request_before_delete {
                let state_clone = state.clone();
                actix_rt::spawn(async move {
                    if let Some(id) = request.id {
                        EventService::broadcast_deleted(
                            &state_clone,
                            "join_school_request",
                            &id.to_hex(),
                            &request,
                        )
                        .await;
                    }
                });
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Join request deleted successfully"
            }))
        }
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Expire old join requests (admin only)
#[post("/admin/expire-old")]
async fn expire_old_join_requests(
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    match expire_old_requests_handler(web::Data::new(controller)).await {
        Ok(response) => response,
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Cleanup expired join requests (admin only)
#[delete("/admin/cleanup-expired/{older_than_days}")]
async fn cleanup_expired_join_requests(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<i64>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = role_guard::check_admin_staff_or_teacher(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let controller = create_join_school_request_controller(&state);
    match cleanup_expired_requests_handler(web::Data::new(controller), path).await {
        Ok(response) => response,
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Count join requests by status
#[get("/stats/count-by-status/{status}")]
async fn count_join_requests_by_status(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let status_str = path.into_inner();

    let status = match status_str.to_lowercase().as_str() {
        "pending" => crate::domain::join_school_request::JoinStatus::Pending,
        "accepted" => crate::domain::join_school_request::JoinStatus::Accepted,
        "rejected" => crate::domain::join_school_request::JoinStatus::Rejected,
        "expired" => crate::domain::join_school_request::JoinStatus::Expired,
        "cancelled" => crate::domain::join_school_request::JoinStatus::Cancelled,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid status. Must be 'pending', 'accepted', 'rejected', 'expired', or 'cancelled'".to_string(),
            })
        }
    };

    match controller.join_request_repo.count_by_status(status).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Count pending join requests by school
#[get("/stats/count-pending/{school_id}")]
async fn count_pending_join_requests_by_school(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);
    let school_id = IdType::String(path.into_inner());

    match controller
        .join_request_repo
        .count_pending_by_school(&school_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Check if user has pending request for school
#[get("/check-pending/{school_id}/{email}")]
async fn check_pending_join_request(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (school_id, email) = path.into_inner();
    let controller = create_join_school_request_controller(&state);
    let school_id = IdType::String(school_id);

    match controller
        .join_request_repo
        .has_pending_request(&email, &school_id)
        .await
    {
        Ok(has_pending) => HttpResponse::Ok().json(serde_json::json!({
            "has_pending": has_pending,
            "email": email,
            "school_id": school_id.as_string()
        })),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join request statistics
#[get("/stats/summary")]
async fn get_join_request_statistics(state: web::Data<AppState>) -> impl Responder {
    let controller = create_join_school_request_controller(&state);

    // Get counts for all statuses
    let pending_count = controller
        .join_request_repo
        .count_by_status(crate::domain::join_school_request::JoinStatus::Pending)
        .await
        .unwrap_or(0);
    let accepted_count = controller
        .join_request_repo
        .count_by_status(crate::domain::join_school_request::JoinStatus::Accepted)
        .await
        .unwrap_or(0);
    let rejected_count = controller
        .join_request_repo
        .count_by_status(crate::domain::join_school_request::JoinStatus::Rejected)
        .await
        .unwrap_or(0);
    let expired_count = controller
        .join_request_repo
        .count_by_status(crate::domain::join_school_request::JoinStatus::Expired)
        .await
        .unwrap_or(0);
    let cancelled_count = controller
        .join_request_repo
        .count_by_status(crate::domain::join_school_request::JoinStatus::Cancelled)
        .await
        .unwrap_or(0);

    let total_count =
        pending_count + accepted_count + rejected_count + expired_count + cancelled_count;

    HttpResponse::Ok().json(serde_json::json!({
        "total": total_count,
        "pending": pending_count,
        "accepted": accepted_count,
        "rejected": rejected_count,
        "expired": expired_count,
        "cancelled": cancelled_count
    }))
}

/// Get my pending join requests (for authenticated user)
#[get("/my/pending")]
async fn get_my_pending_join_requests(
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let controller = create_join_school_request_controller(&state);

    match controller
        .join_request_repo
        .find_pending_with_relations_by_email(&logged_user.email)
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get my join requests (all statuses for authenticated user)
#[get("/my/requests")]
async fn get_my_join_requests(
    user: web::ReqData<AuthUserDto>,
    state: web::Data<AppState>,
) -> impl Responder {
    let logged_user = user.into_inner();
    let controller = create_join_school_request_controller(&state);

    match controller
        .join_request_repo
        .find_by_email_and_status_with_relations(&logged_user.email, None)
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join requests for a specific class
#[get("/class/{class_id}")]
async fn get_join_requests_by_class(
    class_id: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);

    match controller
        .get_join_requests_by_class(&IdType::String(class_id.into_inner()))
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get pending join requests for a specific class
#[get("/class/{class_id}/pending")]
async fn get_pending_join_requests_by_class(
    class_id: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let controller = create_join_school_request_controller(&state);

    match controller
        .get_pending_join_requests_by_class(&IdType::String(class_id.into_inner()))
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Get join requests for a specific school and class
#[get("/school/{school_id}/class/{class_id}")]
async fn get_join_requests_by_school_and_class(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (school_id, class_id) = path.into_inner();
    let controller = create_join_school_request_controller(&state);

    match controller
        .get_join_requests_by_school_and_class(
            &IdType::String(school_id),
            &IdType::String(class_id),
        )
        .await
    {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(e) => HttpResponse::BadRequest().json(ReqErrModel { message: e.message }),
    }
}

/// Helper function to create controller instance
pub fn create_join_school_request_controller(
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

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/join-school-requests")
            // Public routes (read-only, some with basic auth)
            .service(get_join_requests_by_class) // GET /join-requests/class/{class_id} - Get all join requests for a specific class
            .service(get_pending_join_requests_by_class) // GET /join-requests/class/{class_id}/pending - Get pending join requests for a specific class
            .service(get_join_requests_by_school_and_class) // GET /join-requests/school/{school_id}/class/{class_id} - Get join requests for specific school and class combination
            .service(get_all_join_requests) // GET /join-school-requests - Get all join requests with filtering
            .service(get_all_join_requests_with_relations) // GET /join-school-requests/with-relations - Get all join requests with relations
            .service(get_join_request_by_id) // GET /join-school-requests/{id} - Get join request by ID
            .service(get_join_request_by_id_with_relations) // GET /join-school-requests/{id}/with-relations - Get join request by ID with relations
            .service(get_join_requests_by_email) // GET /join-school-requests/email/{email} - Get join requests by email
            .service(get_join_requests_by_school_id) // GET /join-school-requests/school/{school_id} - Get join requests by school ID
            .service(get_pending_join_requests_by_school_id) // GET /join-school-requests/school/{school_id}/pending - Get pending join requests by school ID
            .service(get_join_requests_by_user_id) // GET /join-school-requests/user/{user_id} - Get join requests by invited user ID
            .service(get_join_requests_by_status) // GET /join-school-requests/status/{status} - Get join requests by status
            .service(count_join_requests_by_status) // GET /join-school-requests/stats/count-by-status/{status} - Count join requests by status
            .service(count_pending_join_requests_by_school) // GET /join-school-requests/stats/count-pending/{school_id} - Count pending join requests by school
            .service(check_pending_join_request) // GET /join-school-requests/check-pending/{school_id}/{email} - Check if user has pending request for school
            .service(get_join_request_statistics) // GET /join-school-requests/stats/summary - Get join request statistics summary
            // Protected routes (require JWT)
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_join_request) // POST /join-school-requests - Create new join request (Admin/Staff/Teacher only)
            .service(create_bulk_join_requests) // POST /join-school-requests/bulk - Create multiple join requests (Admin/Staff/Teacher only)
            .service(accept_join_request) // PUT /join-school-requests/{id}/accept - Accept a join request (User for own request or Admin/Staff)
            .service(reject_join_request) // PUT /join-school-requests/{id}/reject - Reject a join request (User for own request or Admin/Staff)
            .service(cancel_join_request) // PUT /join-school-requests/{id}/cancel - Cancel a join request (Sender or Admin/Staff)
            .service(update_join_request_expiration) // PUT /join-school-requests/{id}/expiration - Update request expiration (Admin/Staff/Teacher only)
            .service(bulk_respond_to_join_requests) // PUT /join-school-requests/bulk/respond - Bulk respond to join requests (Admin/Staff/Teacher only)
            .service(delete_join_request) // DELETE /join-school-requests/{id} - Delete join request (Admin/Staff/Teacher only)
            // Personal routes (for authenticated user)
            .service(get_my_pending_join_requests) // GET /join-school-requests/my/pending - Get my pending join requests
            .service(get_my_join_requests) // GET /join-school-requests/my/requests - Get all my join requests
            // Admin-only routes
            .service(expire_old_join_requests) // POST /join-school-requests/admin/expire-old - Expire old join requests (Admin only)
            .service(cleanup_expired_join_requests), // DELETE /join-school-requests/admin/cleanup-expired/{older_than_days} - Cleanup expired join requests (Admin only)
    );
}
