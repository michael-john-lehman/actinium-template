use tokio::time::sleep;
use std::time::Instant;
use std::time::Duration;
use std::future::ready;
use std::future::Ready;
use actix_web::Error;
use actix_web::dev::forward_ready;
use actix_web::dev::Service;
use actix_web::dev::Transform;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use futures_util::future::LocalBoxFuture;

pub struct Minimum {
    duration: Duration,
    client_error: bool,
    success: bool,
    redirect: bool,
    infomational: bool,
    server_error: bool,
    function: Option<fn(&actix_web::http::StatusCode) -> bool>
}

impl Minimum {

    pub fn default() -> Self {
        Self {
            duration: Duration::from_secs(3),
            client_error: false,
            server_error: false,
            success: false,
            redirect: false,
            infomational: false,
            function: None
        }
    }
    
    pub fn client_error(mut self, client_error: bool) -> Self {
        self.client_error = client_error;
        self
    }

    pub fn success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn redirect(mut self, redirect: bool) -> Self {
        self.redirect = redirect;
        self
    }

    pub fn infomational(mut self, infomational: bool) -> Self {
        self.infomational = infomational;
        self
    }

    pub fn server_error(mut self, server_error: bool) -> Self {
        self.server_error = server_error;
        self
    }

    pub fn function(mut self, function: fn(&actix_web::http::StatusCode) -> bool) -> Self {
        self.function = Some(function);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

}

impl<S, B> Transform<S, ServiceRequest> for Minimum
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MinimumMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MinimumMiddleware { 
            service, 
            duration: self.duration,
            client_error: self.client_error,
            server_error: self.server_error,
            success: self.success,
            redirect: self.redirect,
            infomational: self.infomational,
            function: self.function
        }))
    }
}

pub struct MinimumMiddleware<S> {
    duration: Duration,
    client_error: bool,
    success: bool,
    redirect: bool,
    infomational: bool,
    server_error: bool,
    function: Option<fn(&actix_web::http::StatusCode) -> bool>,
    service: S,
}

impl<S, B> Service<ServiceRequest> for MinimumMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let duration = self.duration;
        let success = self.success;
        let client_error = self.client_error;
        let redirect = self.redirect;
        let infomational = self.infomational;
        let server_error = self.server_error;
        let function = self.function;
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let status = res.status();
            let delay = if success && status.is_success() {
                true
            } else if client_error && status.is_client_error() {
                true
            } else if redirect && status.is_redirection() {
                true
            } else if infomational && status.is_informational() {
                true
            } else if server_error && status.is_server_error() {
                true
            } else if let Some(function) = function {
                function(&status)
            } else {
                false
            };
            if delay {
                let elapsed = start.elapsed();
                if let Some(delay) = duration.checked_sub(elapsed) {
                    sleep(delay).await
                }
            }
            Ok(res)
        })
    }
    
}
