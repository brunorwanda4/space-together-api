use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

use crate::config::mongo_manager::MongoManager;

pub struct TenantMiddleware {
    pub mongo: MongoManager,
}

impl TenantMiddleware {
    pub fn new(mongo: MongoManager) -> Self {
        Self { mongo }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TenantMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TenantMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TenantMiddlewareService {
            service: Rc::new(service),
            mongo: self.mongo.clone(),
        })
    }
}

pub struct TenantMiddlewareService<S> {
    service: Rc<S>,
    mongo: MongoManager,
}

impl<S, B> Service<ServiceRequest> for TenantMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);
        let mongo = self.mongo.clone();

        Box::pin(async move {
            // 1) Try header X-School-ID
            let maybe_school_id = req
                .headers()
                .get("x-school-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            // 2) Fallback: extract subdomain from host (localhost handling)
            let school_id = if let Some(id) = maybe_school_id {
                id
            } else {
                // host like st-theresa.space-together.app
                let host = req.connection_info().host().to_string();
                let sub = host.split('.').next().unwrap_or("").to_string();
                // optional: map subdomain to school id by looking up main_db schools collection
                // if sub is "localhost", skip
                sub
            };

            // if school_id is empty or equals "localhost", skip and continue (global endpoints)
            if !school_id.is_empty() && school_id != "localhost" {
                // compute db name
                let db_name = mongo.school_db_name_from_id(&school_id);
                let db_handle = mongo.get_db(&db_name);
                // attach db_handle and school_id to request extensions
                req.extensions_mut().insert(db_handle);
                req.extensions_mut().insert(school_id);
            }

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}
