use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use mongodb::Database;

use crate::{
    domain::auth_user::AuthUserDto,
    domain::subjects::subject_learning_material::{
        SubjectLearningMaterial, SubjectMaterialType, UpdateSubjectLearningMaterial,
    },
    models::{id_model::IdType, request_error_model::ReqErrModel},
    repositories::subjects::subject_learning_material_repo::SubjectLearningMaterialRepo,
    services::subjects::subject_learning_material_service::SubjectLearningMaterialService,
};

#[get("")]
async fn get_all_materials(db: web::Data<Database>) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    match service.get_all_materials().await {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[get("/{id}")]
async fn get_material_by_id(path: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    let material_id = IdType::from_string(path.into_inner());

    match service.get_material_by_id(&material_id).await {
        Ok(material) => HttpResponse::Ok().json(material),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}")]
async fn get_materials_by_subject_id(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service.get_materials_by_subject_id(&subject_id).await {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}/active")]
async fn get_active_materials_by_subject_id(
    path: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    let subject_id = IdType::from_string(path.into_inner());

    match service
        .get_active_materials_by_subject_id(&subject_id)
        .await
    {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}/type/{material_type}")]
async fn get_materials_by_type_and_subject(
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    let (subject_id_str, material_type_str) = path.into_inner();
    let subject_id = IdType::from_string(subject_id_str);

    // Parse material type from string
    let material_type = match material_type_str.to_lowercase().as_str() {
        "book" => SubjectMaterialType::Book,
        "article" => SubjectMaterialType::Article,
        "video" => SubjectMaterialType::Video,
        "note" => SubjectMaterialType::Note,
        "externallink" => SubjectMaterialType::ExternalLink,
        "document" => SubjectMaterialType::Document,
        _ => {
            return HttpResponse::BadRequest().json(ReqErrModel {
                message: "Invalid material type".to_string(),
            })
        }
    };

    match service
        .get_materials_by_type_and_subject(&material_type, &subject_id)
        .await
    {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[get("/subject/{subject_id}/search/{search_term}")]
async fn search_materials_by_title(
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> impl Responder {
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    let (subject_id_str, search_term) = path.into_inner();
    let subject_id = IdType::from_string(subject_id_str);

    match service
        .search_materials_by_title(&subject_id, &search_term)
        .await
    {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(message) => HttpResponse::NotFound().json(ReqErrModel { message }),
    }
}

#[post("")]
async fn create_material(
    user: web::ReqData<AuthUserDto>,
    data: web::Json<SubjectLearningMaterial>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    match service.create_material(data.into_inner()).await {
        Ok(material) => HttpResponse::Created().json(material),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}")]
async fn update_material(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<String>,
    data: web::Json<UpdateSubjectLearningMaterial>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let material_id = IdType::from_string(path.into_inner());
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    match service
        .update_material(&material_id, data.into_inner())
        .await
    {
        Ok(material) => HttpResponse::Ok().json(material),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[put("/{id}/status/{is_active}")]
async fn toggle_material_status(
    user: web::ReqData<AuthUserDto>,
    path: web::Path<(String, bool)>,
    db: web::Data<Database>,
) -> impl Responder {
    let logged_user = user.into_inner();

    if let Err(err) = crate::guards::role_guard::check_admin(&logged_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "message": err.to_string()
        }));
    }

    let (material_id_str, is_active) = path.into_inner();
    let material_id = IdType::from_string(material_id_str);
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    match service
        .toggle_material_status(&material_id, is_active)
        .await
    {
        Ok(material) => HttpResponse::Ok().json(material),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

#[delete("/{id}")]
async fn delete_material(
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

    let material_id = IdType::from_string(path.into_inner());
    let repo = SubjectLearningMaterialRepo::new(db.get_ref());
    let service = SubjectLearningMaterialService::new(&repo);

    match service.delete_material(&material_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Learning material deleted successfully"
        })),
        Err(message) => HttpResponse::BadRequest().json(ReqErrModel { message }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subject-learning-materials")
            // Public routes
            .service(get_all_materials) // GET /subject-learning-materials
            .service(get_material_by_id) // GET /subject-learning-materials/{id}
            .service(get_materials_by_subject_id) // GET /subject-learning-materials/subject/{subject_id}
            .service(get_active_materials_by_subject_id) // GET /subject-learning-materials/subject/{subject_id}/active
            .service(get_materials_by_type_and_subject) // GET /subject-learning-materials/subject/{subject_id}/type/{material_type}
            .service(search_materials_by_title) // GET /subject-learning-materials/subject/{subject_id}/search/{search_term}
            // Protected routes
            .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
            .service(create_material) // POST /subject-learning-materials
            .service(update_material) // PUT /subject-learning-materials/{id}
            .service(toggle_material_status) // PUT /subject-learning-materials/{id}/status/{is_active}
            .service(delete_material), // DELETE /subject-learning-materials/{id}
    );
}
