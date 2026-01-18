i think it's better if every school have they on the endpoint to listen example is:
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/school/timetables")
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            .service(create_timetable)
    );
}

example is:
(
#[post("")]
async fn create_comment(
    req: HttpRequest,
    user: web::ReqData<AuthUserDto>,
    data: web::Json<Comment>,
    state: web::Data<AppState>,
) -> impl Responder {
    let _logged_user = user.into_inner();
    let db = get_database(&req, &state);
    let service = CommentService::new(&db);

    match service.create(data.into_inner()).await {
        Ok(item) => {
            let cloned = item.clone();
            let state_clone = state.clone();

            actix_rt::spawn(async move {
                if let Some(id) = cloned.id {
                    EventService::broadcast_created(&state_clone, "comment", &id.to_hex(), &cloned)
                        .await;
                }
            });

            HttpResponse::Created().json(item)
        }
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

fn blueprint(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_comments)
        .service(
            web::scope("")
                .wrap(crate::middleware::jwt_middleware::JwtMiddleware)
                .service(create_comment)
        );
}

pub fn init(cfg: &mut web::ServiceConfig) {
    crate::utils::route_utils::mount_dual_routes(cfg, "comments", blueprint);
}
)

also this is  crate::utils::route_utils::mount_dual_routes: (use actix_web::web;

pub fn mount_dual_routes<F>(cfg: &mut web::ServiceConfig, path: &str, register_handlers: F)
where
    F: Fn(&mut web::ServiceConfig) + Copy,
{
    cfg.service(
        web::scope(&format!("/school/{}", path))
            .wrap(crate::middleware::school_token_middleware::SchoolTokenMiddleware)
            .configure(register_handlers),
    );

    cfg.service(web::scope(&format!("/{}", path)).configure(register_handlers));
}
)

and school token schema is school_token_model.rs:

(use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::{
    common_details::RelatedUser,
    school::{AffiliationType, SchoolType},
};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolToken {
    pub id: String,
    pub creator_id: Option<String>,
    pub name: String,
    pub username: String,
    pub logo: Option<String>,
    pub school_type: Option<SchoolType>, // or SchoolType if you want enum in token
    pub affiliation: Option<AffiliationType>, // optional string form of affiliation
    pub database_name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub member: Option<RelatedUser>,
    pub exp: usize,
    pub iat: usize,
}
)