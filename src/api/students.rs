use actix_web::{get, post, web, HttpResponse};
use mongodb::Database;
use serde::Deserialize;

use crate::{domain::student::Student, services::student_service::StudentService};

#[derive(Deserialize)]
pub struct CreateStudent {
    pub name: String,
    pub email: String,
    pub age: i32,
}

#[post("/students")]
async fn create_student(db: web::Data<Database>, data: web::Json<CreateStudent>) -> HttpResponse {
    let service = StudentService::new(db.get_ref().clone());
    let student = Student {
        id: None,
        name: data.name.clone(),
        email: data.email.clone(),
        age: data.age,
    };

    match service.create_student(student).await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({ "status": "success" })),
        Err(err) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": err.to_string() })),
    }
}

#[get("/students")]
async fn list_students(db: web::Data<Database>) -> HttpResponse {
    let service = StudentService::new(db.get_ref().clone());
    match service.list_students().await {
        Ok(students) => HttpResponse::Ok().json(students),
        Err(err) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": err.to_string() })),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(create_student);
    cfg.service(list_students);
}
