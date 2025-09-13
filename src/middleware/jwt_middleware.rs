use crate::utils::jwt::verify_jwt;
use actix_web::body::EitherBody;
use actix_web::{
    dev::{forward_ready, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JwtMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct JwtMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for JwtMiddlewareMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // Extract Authorization header
            let token = if let Some(auth_header) = req.headers().get("Authorization") {
                let auth_str = auth_header.to_str().unwrap_or("");
                if auth_str.starts_with("Bearer ") {
                    Some(auth_str.trim_start_matches("Bearer ").to_string())
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(token) = token {
                if let Some(claims) = verify_jwt(&token) {
                    // attach user info to request extensions
                    req.extensions_mut().insert(claims.user);
                    let res = svc.call(req).await?.map_into_left_body();
                    return Ok(res);
                }
            }

            // unauthorized
            let res = req.into_response(
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "Unauthorized"}))
                    .map_into_right_body(),
            );
            Ok(res)
        })
    }
}
