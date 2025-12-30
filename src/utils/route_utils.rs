use actix_web::web;

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
