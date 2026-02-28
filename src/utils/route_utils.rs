use actix_web::web;

/// Mount routes that work both with and without school context
/// - /school/{path} - School-specific route (requires SchoolTokenMiddleware)
/// - /{path} - General route (works for both school and non-school contexts)
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


/// Mount routes for messaging/conversations that automatically detect school context from headers
/// Uses OptionalSchoolTokenMiddleware to check for school token without requiring it
/// - If school token exists in header → routes to school database
/// - If no school token → routes to main database
pub fn mount_messaging_routes<F>(cfg: &mut web::ServiceConfig, path: &str, register_handlers: F)
where
    F: Fn(&mut web::ServiceConfig) + Copy,
{
    cfg.service(
        web::scope(&format!("/{}", path))
            .wrap(crate::middleware::school_token_middleware::OptionalSchoolTokenMiddleware)
            .configure(register_handlers),
    );
}
