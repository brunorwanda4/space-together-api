use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;
use std::collections::HashMap;

use crate::{
    domain::auth_user::AuthUserDto,
    domain::subjects::subject_category::SubjectTypeFor,
    domain::subjects::subject_grading_schemes::{
        SubjectGradingScheme, SubjectGradingType, UpdateSubjectGradingScheme,
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::subject_grading_schemes_repo::SubjectGradingSchemesRepo,
    services::subjects::subject_grading_schemes_service::SubjectGradingSchemesService,
};

#[get("")]
async fn get_all_schemes(db: web::Data<Database>) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    match service.get_all_schemes().await {
        Ok(schemes) => HttpResponse::Ok().json(schemes),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_scheme_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let scheme_id = IdType::from_string(path.into_inner());

    match service.get_scheme_by_id(&scheme_id).await {
        Ok(scheme) => HttpResponse::Ok().json(scheme),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}")]
async fn get_scheme_by_subject_id(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_scheme_by_main_subject_id(&subject_id).await {
        Ok(scheme) => HttpResponse::Ok().json(scheme),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}/role/{role}")]
async fn get_scheme_by_subject_and_role(
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let (subject_id_str, role_str) = path.into_inner();
    let subject_id = IdType::from_string(subject_id_str);

    // Parse role from string
    let role = match role_str.to_lowercase().as_str() {
        "MainSubject" => SubjectTypeFor::MainSubject,
        "ClassSubject" => SubjectTypeFor::ClassSubject,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid role type".to_string(),
            })
        }
    };

    match service
        .get_scheme_by_subject_and_role(&subject_id, &role)
        .await
    {
        Ok(scheme) => HttpResponse::Ok().json(scheme),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/type/{scheme_type}")]
async fn get_schemes_by_type(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let scheme_type_str = path.into_inner();

    // Parse scheme type from string
    let scheme_type = match scheme_type_str.to_lowercase().as_str() {
        "lettergrade" => SubjectGradingType::LetterGrade,
        "percentage" => SubjectGradingType::Percentage,
        "points" => SubjectGradingType::Points,
        "passfail" => SubjectGradingType::PassFail,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid scheme type".to_string(),
            })
        }
    };

    match service.get_schemes_by_type(&scheme_type).await {
        Ok(schemes) => HttpResponse::Ok().json(schemes),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("/{id}/calculate-grade")]
async fn calculate_grade(
    path: web::Path<String>,
    data: web::Json<HashMap<String, f32>>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let scheme_id = IdType::from_string(path.into_inner());

    match service
        .calculate_grade(&scheme_id, &data.into_inner())
        .await
    {
        Ok((grade, score)) => HttpResponse::Ok().json(serde_json::json!({
            "grade": grade,
            "score": score
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/{id}/check-passing")]
async fn check_passing_grade(
    path: web::Path<String>,
    data: web::Json<HashMap<String, String>>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let scheme_id = IdType::from_string(path.into_inner());

    let grade = data.get("grade").ok_or_else(|| {
        HttpResponse::BadRequest().json(ReqErrModel {
            message: "Grade field is required".to_string(),
        })
    });

    let grade = match grade {
        Ok(grade) => grade,
        Err(response) => return response,
    };

    match service.is_passing_grade(&scheme_id, grade).await {
        Ok(is_passing) => HttpResponse::Ok().json(serde_json::json!({
            "is_passing": is_passing,
            "grade": grade
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_scheme(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SubjectGradingScheme>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    match service.create_scheme(data.into_inner()).await {
        Ok(scheme) => HttpResponse::Created().json(scheme),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[post("/subject/{subject_id}/default")]
async fn create_default_scheme(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    // Parse the logged user's id (String) into a MongoDB ObjectId; return BadRequest if invalid
    let user_oid = match mongodb::bson::oid::ObjectId::parse_str(&logged_user.id) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid user id".to_string(),
            });
        }
    };

    match service
        .get_or_create_default_scheme(&subject_id, Some(user_oid))
        .await
    {
        Ok(scheme) => HttpResponse::Created().json(scheme),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_scheme(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSubjectGradingScheme>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let scheme_id = IdType::from_string(path.into_inner());
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    match service.update_scheme(&scheme_id, data.into_inner()).await {
        Ok(scheme) => HttpResponse::Ok().json(scheme),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_scheme(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let scheme_id = IdType::from_string(path.into_inner());
    let repo = SubjectGradingSchemesRepo::new(db.get_ref());
    let service = SubjectGradingSchemesService::new(&repo);

    match service.delete_scheme(&scheme_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Grading scheme deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subject-grading-schemes")
            // Public routes
            .service(get_all_schemes) // GET /subject-grading-schemes
            .service(get_scheme_by_id) // GET /subject-grading-schemes/{id}
            .service(get_scheme_by_subject_id) // GET /subject-grading-schemes/subject/{subject_id}
            .service(get_scheme_by_subject_and_role) // GET /subject-grading-schemes/subject/{subject_id}/role/{role}
            .service(get_schemes_by_type) // GET /subject-grading-schemes/type/{scheme_type}
            .service(calculate_grade) // POST /subject-grading-schemes/{id}/calculate-grade
            .service(check_passing_grade) // POST /subject-grading-schemes/{id}/check-passing
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_scheme) // POST /subject-grading-schemes
            .service(create_default_scheme) // POST /subject-grading-schemes/subject/{subject_id}/default
            .service(update_scheme) // PUT /subject-grading-schemes/{id}
            .service(delete_scheme), // DELETE /subject-grading-schemes/{id}
    );
}
