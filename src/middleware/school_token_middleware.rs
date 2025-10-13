use crate::utils::school_token::verify_school_token;
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

pub struct SchoolTokenMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SchoolTokenMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = SchoolTokenMiddlewareImpl<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SchoolTokenMiddlewareImpl {
            service: Rc::new(service),
        })
    }
}

pub struct SchoolTokenMiddlewareImpl<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SchoolTokenMiddlewareImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // Extract `School-Token` header
            let token_opt = req
                .headers()
                .get("School-Token")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            if let Some(token) = token_opt {
                if let Some(claims) = verify_school_token(&token) {
                    req.extensions_mut().insert(claims);
                    let res = svc.call(req).await?.map_into_left_body();
                    return Ok(res);
                }
            }

            let res = req.into_response(
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({"message": "Invalid or missing school token 😣"}))
                    .map_into_right_body(),
            );
            Ok(res)
        })
    }
}
