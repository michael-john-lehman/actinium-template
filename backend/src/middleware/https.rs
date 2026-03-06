use actix_web::*;
use std::future::{ready, Ready};
use actix_web::body::{EitherBody, BoxBody};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;

fn is_https(req: &ServiceRequest) -> bool {
    use actix_web::http::uri::Scheme;
    if let Some(scheme) = req.request().uri().scheme() {
        *scheme == Scheme::HTTPS
    } else {
        false
    }
}

fn insert_location(builder: &mut HttpResponseBuilder, request: &HttpRequest) {
    let info = request.connection_info();
    let host = info.host();
    let uri = request.uri().to_string();
    builder.insert_header((http::header::LOCATION, ["https://", host, uri.as_str()].concat()));
}

pub struct RedirectHTTPS {
    enabled: bool
}

impl RedirectHTTPS {

    pub fn enabled(enabled: bool) -> Self {
        Self { enabled }
    }

}

pub struct RedirectHTTPSMiddleware<S> {
    enabled: bool,
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for RedirectHTTPS
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = RedirectHTTPSMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RedirectHTTPSMiddleware { service, enabled: self.enabled }))
    }
}

impl<S, B> Service<ServiceRequest> for RedirectHTTPSMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if self.enabled && !is_https(&req) {
            log::info!("not https");
            return Box::pin(async move {
                let (request, _) = req.into_parts();
                let mut builder = HttpResponseBuilder::new(http::StatusCode::MOVED_PERMANENTLY);
                insert_location(&mut builder, &request);
                let res = ServiceResponse::new(request, builder.finish()).map_into_boxed_body();
                Ok(res.map_into_right_body())
            });
        }
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
   
}
