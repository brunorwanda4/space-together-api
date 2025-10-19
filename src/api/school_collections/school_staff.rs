use std::str::FromStr;

use actix_web::{delete, get, post, put, web, HttpMessage, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

use crate::{
    config::state::AppState,
    domain::school_staff::{
        BulkIdsRequest, BulkTagsRequest, BulkUpdateActiveStatusRequest, PrepareStaffRequest,
        SchoolStaff, SchoolStaffType, UpdateSchoolStaff,
    },
    models::{
        api_request_model::RequestQuery, id_model::IdType, request_error_model::ReqErrModel,
        school_token_model::SchoolToken,
    },
    repositories::school_staff_repo::SchoolStaffRepo,
    services::{event_service::EventService, school_staff_service::SchoolStaffService},
};

#[get("")]
async fn get_all_school_staff(
    req: actix_web::HttpRequest,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Get school claims from extensions (set by SchoolTokenMiddleware)
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    // Use school database from claims
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .get_all_school_staff(query.filter.clone(), query.limit, query.skip)
        .await
    {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/active")]
async fn get_active_school_staff(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_active_school_staff().await {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_school_staff_by_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_school_staff_by_id(&staff_id).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/user/{user_id}")]
async fn get_school_staff_by_user_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let user_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_school_staff_by_user_id(&user_id).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/email/{email}")]
async fn get_school_staff_by_email(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let email = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_school_staff_by_email(&email).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_school_staff(
    req: actix_web::HttpRequest,
    data: web::Json<SchoolStaff>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // Set the school_id from the token to ensure consistency
    let mut staff_data = data.into_inner();
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    staff_data.school_id = Some(school_id);

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .create_school_staff_with_events(staff_data, &state)
        .await
    {
        Ok(staff) => HttpResponse::Created().json(staff),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_school_staff(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSchoolStaff>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .update_school_staff_with_events(&staff_id, data.into_inner(), &state)
        .await
    {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_school_staff(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .delete_school_staff_with_events(&staff_id, &state)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "School staff deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[derive(Deserialize)]
struct StaffCountQuery {
    r#type: Option<SchoolStaffType>,
    is_active: Option<bool>,
}

#[get("/stats/count")]
async fn count_school_staff(
    query: web::Query<StaffCountQuery>,
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .count_staff_by_school_id(
            &IdType::from_string(claims.id.clone()),
            query.r#type.clone(),
            query.is_active,
        )
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/with-details")]
async fn get_all_school_staff_with_details(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_all_school_staff_with_relations().await {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}/with-details")]
async fn get_school_staff_by_id_with_details(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .get_school_staff_by_id_with_relations(&staff_id)
        .await
    {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/creator/{creator_id}")]
async fn get_school_staff_by_creator_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let creator_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_school_staff_by_creator_id(&creator_id).await {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/type/{staff_type}")]
async fn get_school_staff_by_type(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_type_str = path.into_inner();
    let staff_type = match staff_type_str.as_str() {
        "director" => SchoolStaffType::Director,
        "head_of_studies" => SchoolStaffType::HeadOfStudies,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid staff type. Must be 'director' or 'head_of_studies'".to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_school_staff_by_type(staff_type).await {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/school/{school_id}/type/{staff_type}")]
async fn get_school_staff_by_school_and_type(
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let (school_id_str, staff_type_str) = path.into_inner();
    let school_id = IdType::from_string(school_id_str);

    let staff_type = match staff_type_str.as_str() {
        "director" => SchoolStaffType::Director,
        "head_of_studies" => SchoolStaffType::HeadOfStudies,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid staff type. Must be 'director' or 'head_of_studies'".to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .get_school_staff_by_school_and_type(&school_id, staff_type)
        .await
    {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[put("/{id}/merged")]
async fn update_school_staff_merged(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<UpdateSchoolStaff>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .update_school_staff_merged(&staff_id, data.into_inner())
        .await
    {
        Ok(staff) => {
            // Broadcast updated staff event
            let staff_clone = staff.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = staff_clone.id {
                    EventService::broadcast_updated(
                        &state_clone,
                        "school_staff",
                        &id.to_hex(),
                        &staff_clone,
                    )
                    .await;
                }
            });

            HttpResponse::Ok().json(staff)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-creator/{creator_id}")]
async fn count_school_staff_by_creator_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let creator_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.count_school_staff_by_creator_id(&creator_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/stats/count-by-type/{staff_type}")]
async fn count_school_staff_by_type(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let staff_type_str = path.into_inner();
    let staff_type = match staff_type_str.as_str() {
        "director" => SchoolStaffType::Director,
        "head_of_studies" => SchoolStaffType::HeadOfStudies,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid staff type. Must be 'director' or 'head_of_studies'".to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.count_school_staff_by_type(staff_type).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "count": count })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple staff members for school
#[post("/bulk")]
async fn create_many_school_staff(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<SchoolStaff>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // Set school_id for all staff members
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    let mut staff_with_school = data.into_inner();
    for staff in &mut staff_with_school {
        staff.school_id = Some(school_id);
    }

    // ✅ USE THE EVENT-BASED METHOD
    match service
        .create_many_school_staff_with_events(staff_with_school, &state)
        .await
    {
        Ok(staff_members) => HttpResponse::Created().json(staff_members),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple staff members with validation for school
#[post("/bulk/validation")]
async fn create_many_school_staff_with_validation(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<SchoolStaff>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // Set school_id for all staff members
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to change school id into object id: {}", e),
            });
        }
    };

    let mut staff_with_school = data.into_inner();
    for staff in &mut staff_with_school {
        staff.school_id = Some(school_id);
    }

    match service
        .create_many_school_staff_with_validation(staff_with_school)
        .await
    {
        Ok(staff_members) => {
            let state_clone = state.clone();
            let staff_for_spawn = staff_members.clone();

            actix_rt::spawn(async move {
                for staff in &staff_for_spawn {
                    if let Some(id) = staff.id {
                        EventService::broadcast_created(
                            &state_clone,
                            "school_staff",
                            &id.to_hex(),
                            staff,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Created().json(staff_members)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update active status for multiple staff members
#[put("/bulk/active-status")]
async fn bulk_update_school_staff_active_status(
    req: actix_web::HttpRequest,
    data: web::Json<BulkUpdateActiveStatusRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.bulk_update_active_status(&data.into_inner()).await {
        Ok(staff_members) => {
            let state_clone = state.clone();
            let staff_for_spawn = staff_members.clone();

            actix_rt::spawn(async move {
                for staff in &staff_for_spawn {
                    if let Some(id) = staff.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "school_staff",
                            &id.to_hex(),
                            staff,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(staff_members)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk add tags to multiple staff members
#[put("/bulk/add-tags")]
async fn bulk_add_tags_to_school_staff(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTagsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.bulk_add_tags(&data.into_inner()).await {
        Ok(staff_members) => {
            let state_clone = state.clone();
            let staff_for_spawn = staff_members.clone();

            actix_rt::spawn(async move {
                for staff in &staff_for_spawn {
                    if let Some(id) = staff.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "school_staff",
                            &id.to_hex(),
                            staff,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(staff_members)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk remove tags from multiple staff members
#[put("/bulk/remove-tags")]
async fn bulk_remove_tags_from_school_staff(
    req: actix_web::HttpRequest,
    data: web::Json<BulkTagsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.bulk_remove_tags(&data.into_inner()).await {
        Ok(staff_members) => {
            let state_clone = state.clone();
            let staff_for_spawn = staff_members.clone();

            actix_rt::spawn(async move {
                for staff in &staff_for_spawn {
                    if let Some(id) = staff.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "school_staff",
                            &id.to_hex(),
                            staff,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(staff_members)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Delete multiple staff members
#[delete("/bulk")]
async fn delete_many_school_staff(
    req: actix_web::HttpRequest,
    data: web::Json<BulkIdsRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.delete_many_school_staff(&data.into_inner()).await {
        Ok(deleted_count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully deleted {} staff members", deleted_count)
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get school director
#[get("/director")]
async fn get_school_director(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .get_school_director(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(Some(director)) => HttpResponse::Ok().json(director),
        Ok(None) => HttpResponse::NotFound().json(ReqErrModel {
            message: "Director not found for this school".to_string(),
        }),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get head of studies
#[get("/head-of-studies")]
async fn get_head_of_studies(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .get_head_of_studies(&IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(Some(head)) => HttpResponse::Ok().json(head),
        Ok(None) => HttpResponse::NotFound().json(ReqErrModel {
            message: "Head of studies not found for this school".to_string(),
        }),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if user is staff member of this school
#[get("/check/{user_id}")]
async fn is_user_school_staff(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let user_id = IdType::from_string(path.into_inner());
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .is_user_school_staff(&user_id, &IdType::from_string(claims.id.clone()))
        .await
    {
        Ok(is_staff) => HttpResponse::Ok().json(serde_json::json!({ "is_staff": is_staff })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Check if user has specific staff type in this school
#[get("/check/{user_id}/type/{staff_type}")]
async fn has_staff_type(
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let (user_id_str, staff_type_str) = path.into_inner();
    let user_id = IdType::from_string(user_id_str);

    let staff_type = match staff_type_str.as_str() {
        "director" => SchoolStaffType::Director,
        "head_of_studies" => SchoolStaffType::HeadOfStudies,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid staff type. Must be 'director' or 'head_of_studies'".to_string(),
            })
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service
        .has_staff_type(
            &user_id,
            &IdType::from_string(claims.id.clone()),
            staff_type,
        )
        .await
    {
        Ok(has_type) => HttpResponse::Ok().json(serde_json::json!({ "has_type": has_type })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Create multiple staff members for a specific school
#[post("/school/{school_id}/bulk")]
async fn create_many_school_staff_for_school(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<SchoolStaff>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_id_str = path.into_inner();
    let school_id = IdType::from_string(school_id_str);

    // Verify that the school ID in the path matches the token's school ID
    if school_id.as_string() != claims.id {
        return HttpResponse::Forbidden().json(ReqErrModel {
            message: "Cannot create staff for different school".to_string(),
        });
    }

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // ✅ USE THE EVENT-BASED METHOD (create_many_school_staff_with_events instead of create_many_school_staff_for_school)
    let mut staff_with_school = data.into_inner();
    let school_id_obj = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to parse school id: {}", e),
            });
        }
    };

    for staff in &mut staff_with_school {
        staff.school_id = Some(school_id_obj);
    }

    match service
        .create_many_school_staff_with_events(staff_with_school, &state)
        .await
    {
        Ok(staff_members) => HttpResponse::Created().json(staff_members),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Bulk update multiple school staff members
#[put("/bulk-update")]
async fn update_many_school_staff(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<(String, UpdateSchoolStaff)>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // Convert the request data to the format expected by the service
    let updates: Vec<(IdType, UpdateSchoolStaff)> = data
        .into_inner()
        .into_iter()
        .map(|(id_str, update)| (IdType::from_string(id_str), update))
        .collect();

    match service.update_many_school_staff(updates).await {
        Ok(updated_staff) => {
            let state_clone = state.clone();
            let staff_for_spawn = updated_staff.clone();

            actix_rt::spawn(async move {
                for staff in &staff_for_spawn {
                    if let Some(id) = staff.id {
                        EventService::broadcast_updated(
                            &state_clone,
                            "school_staff",
                            &id.to_hex(),
                            staff,
                        )
                        .await;
                    }
                }
            });

            HttpResponse::Ok().json(updated_staff)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get all staff members for a specific school
#[get("/school/{school_id}")]
async fn get_school_staff_by_school_id(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_id_str = path.into_inner();
    let school_id = IdType::from_string(school_id_str);

    // Verify that the school ID in the path matches the token's school ID
    if school_id.as_string() != claims.id {
        return HttpResponse::Forbidden().json(ReqErrModel {
            message: "Cannot access staff for different school".to_string(),
        });
    }

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    match service.get_school_staff_by_school_id(&school_id).await {
        Ok(staff_members) => HttpResponse::Ok().json(staff_members),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Get all staff members for a specific school with pagination and filtering
#[get("/school/{school_id}/filtered")]
async fn get_school_staff_by_school_id_filtered(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    query: web::Query<RequestQuery>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_id_str = path.into_inner();
    let school_id = IdType::from_string(school_id_str);

    // Verify that the school ID in the path matches the token's school ID
    if school_id.as_string() != claims.id {
        return HttpResponse::Forbidden().json(ReqErrModel {
            message: "Cannot access staff for different school".to_string(),
        });
    }

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // First get all staff for the school, then apply filtering
    match service.get_school_staff_by_school_id(&school_id).await {
        Ok(staff_members) => {
            // Apply client-side filtering if needed, or use repository filtering
            let filtered_staff = if let Some(filter) = &query.filter {
                staff_members
                    .into_iter()
                    .filter(|staff| {
                        staff.name.to_lowercase().contains(&filter.to_lowercase())
                            || staff.email.to_lowercase().contains(&filter.to_lowercase())
                            || staff
                                .tags
                                .iter()
                                .any(|tag| tag.to_lowercase().contains(&filter.to_lowercase()))
                    })
                    .collect()
            } else {
                staff_members
            };

            // Apply pagination
            let skip = query.skip.unwrap_or(0) as usize;
            let limit = query.limit.unwrap_or(50) as usize;

            let paginated_staff: Vec<SchoolStaff> =
                filtered_staff.into_iter().skip(skip).take(limit).collect();

            HttpResponse::Ok().json(paginated_staff)
        }
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare school staff members for bulk creation
/// This endpoint prepares staff data by setting school_id and creator_id without saving to database
#[post("/bulk/prepare")]
async fn prepare_school_staff_for_bulk_creation(
    req: actix_web::HttpRequest,
    data: web::Json<Vec<SchoolStaff>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // Parse school_id from token
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to parse school id: {}", e),
            });
        }
    };

    // Note: creator_id is not available in school token, so we pass None
    // If you need creator_id, you might need to get it from a different source
    match service.prepare_school_staff_for_bulk_creation(
        data.into_inner(),
        Some(school_id),
        None, // creator_id would typically come from user token, not school token
    ) {
        Ok(prepared_staff) => HttpResponse::Ok().json(prepared_staff),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare school staff members for bulk creation with optional creator ID
/// This endpoint allows specifying a creator_id for the prepared staff data
#[post("/bulk/prepare-with-creator/{creator_id}")]
async fn prepare_school_staff_for_bulk_creation_with_creator(
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    data: web::Json<Vec<SchoolStaff>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let creator_id_str = path.into_inner();
    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    // Parse school_id from token
    let school_id = match ObjectId::from_str(&claims.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to parse school id: {}", e),
            });
        }
    };

    // Parse creator_id from path parameter
    let creator_id = match ObjectId::from_str(&creator_id_str) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: format!("Failed to parse creator id: {}", e),
            });
        }
    };

    match service.prepare_school_staff_for_bulk_creation(
        data.into_inner(),
        Some(school_id),
        Some(creator_id),
    ) {
        Ok(prepared_staff) => HttpResponse::Ok().json(prepared_staff),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

/// Prepare school staff members for bulk creation with custom parameters
/// This endpoint allows full control over school_id and creator_id
#[post("/bulk/prepare-custom")]
async fn prepare_school_staff_for_bulk_creation_custom(
    req: actix_web::HttpRequest,
    data: web::Json<PrepareStaffRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match req.extensions().get::<SchoolToken>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "School token required"
            }))
        }
    };

    let school_db = state.db.get_db(&claims.database_name);
    let repo = SchoolStaffRepo::new(&school_db);
    let service = SchoolStaffService::new(&repo);

    let request_data = data.into_inner();

    // Use provided school_id or fall back to token's school_id
    let school_id = if let Some(school_id_str) = request_data.school_id {
        match ObjectId::from_str(&school_id_str) {
            Ok(id) => Some(id),
            Err(_) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: "Invalid school_id provided".to_string(),
                });
            }
        }
    } else {
        // Use school_id from token
        ObjectId::from_str(&claims.id).ok()
    };

    // Parse creator_id if provided
    let creator_id = if let Some(creator_id_str) = request_data.creator_id {
        match ObjectId::from_str(&creator_id_str) {
            Ok(id) => Some(id),
            Err(_) => {
                return HttpResponse::BadRequest().json(ReqErrModel {
                    message: "Invalid creator_id provided".to_string(),
                });
            }
        }
    } else {
        None
    };

    match service.prepare_school_staff_for_bulk_creation(
        request_data.staff_members,
        school_id,
        creator_id,
    ) {
        Ok(prepared_staff) => HttpResponse::Ok().json(prepared_staff),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/staff")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            // =============================================
            // PUBLIC ROUTES (READ-ONLY)
            // =============================================
            // Staff Listing & Retrieval
            .service(get_all_school_staff) // GET    /school/staff - Get all staff members with optional filtering and pagination
            .service(get_all_school_staff_with_details) // GET    /school/staff/with-details - Get all staff members with user and school relations
            .service(get_active_school_staff) // GET    /school/staff/active - Get only active staff members
            .service(get_school_staff_by_id) // GET    /school/staff/{id} - Get staff member by ID
            .service(get_school_staff_by_id_with_details) // GET    /school/staff/{id}/with-details - Get staff member by ID with user and school relations
            .service(get_school_staff_by_user_id) // GET    /school/staff/user/{user_id} - Get staff member by associated user ID
            .service(get_school_staff_by_email) // GET    /school/staff/email/{email} - Get staff member by email address
            .service(get_school_staff_by_creator_id) // GET    /school/staff/creator/{creator_id} - Get staff members created by specific user
            .service(get_school_staff_by_type) // GET    /school/staff/type/{staff_type} - Get staff members by type (director/head_of_studies)
            .service(get_school_staff_by_school_and_type) // GET    /school/staff/school/{school_id}/type/{staff_type} - Get staff by school and type combination
            .service(get_school_staff_by_school_id) // GET    /school/staff/school/{school_id} - Get all staff members for specific school
            .service(get_school_staff_by_school_id_filtered) // GET    /school/staff/school/{school_id}/filtered - Get staff for school with filtering & pagination
            // Role-based Access
            .service(get_school_director) // GET    /school/staff/director - Get the director for current school
            .service(get_head_of_studies) // GET    /school/staff/head-of-studies - Get head of studies for current school
            // Permission Checking
            .service(is_user_school_staff) // GET    /school/staff/check/{user_id} - Check if user is staff member of current school
            .service(has_staff_type) // GET    /school/staff/check/{user_id}/type/{staff_type} - Check if user has specific staff role in school
            // Statistics & Analytics
            .service(count_school_staff) // GET    /school/staff/stats/count - Get total count of staff members in school
            .service(count_school_staff_by_creator_id) // GET    /school/staff/stats/count-by-creator/{creator_id} - Count staff created by specific user
            .service(count_school_staff_by_type) // GET    /school/staff/stats/count-by-type/{staff_type} - Count staff by type in school
            // =============================================
            // PROTECTED ROUTES (WRITE OPERATIONS)
            // =============================================
            // Single Staff Operations
            .service(create_school_staff) // POST   /school/staff - Create new staff member
            .service(update_school_staff) // PUT    /school/staff/{id} - Update staff member by ID
            .service(update_school_staff_merged) // PUT    /school/staff/{id}/merged - Update staff member with full data merge
            .service(delete_school_staff) // DELETE /school/staff/{id} - Delete staff member by ID
            // =============================================
            // BULK OPERATIONS
            // =============================================
            // Bulk Creation
            .service(create_many_school_staff) // POST   /school/staff/bulk - Create multiple staff members
            .service(create_many_school_staff_with_validation) // POST   /school/staff/bulk/validation - Create multiple staff with comprehensive validation
            .service(create_many_school_staff_for_school) // POST   /school/staff/school/{school_id}/bulk - Create multiple staff for specific school
            // Bulk Updates
            .service(update_many_school_staff) // PUT    /school/staff/bulk-update - Update multiple staff members in single request
            .service(bulk_update_school_staff_active_status) // PUT    /school/staff/bulk/active-status - Bulk update active status for multiple staff
            .service(bulk_add_tags_to_school_staff) // PUT    /school/staff/bulk/add-tags - Bulk add tags to multiple staff members
            .service(bulk_remove_tags_from_school_staff) // PUT    /school/staff/bulk/remove-tags - Bulk remove tags from multiple staff members
            // Bulk Deletion
            .service(delete_many_school_staff) // DELETE /school/staff/bulk - Delete multiple staff members by IDs
            // PREPARATION ENDPOINTS - Add the preparation endpoints here
            .service(prepare_school_staff_for_bulk_creation) // POST   /school/staff/bulk/prepare - Prepare staff data for bulk creation (auto-set school_id from token)
            .service(prepare_school_staff_for_bulk_creation_with_creator) // POST   /school/staff/bulk/prepare-with-creator/{creator_id} - Prepare staff data with specific creator ID
            .service(prepare_school_staff_for_bulk_creation_custom), // POST   /school/staff/bulk/prepare-custom - Prepare staff data with full control over school_id and creator_id
    );
}
