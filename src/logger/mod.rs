use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use chrono::Utc;
use futures::future::{ok, Ready};
use futures::Future;
use log::error;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Logger;

impl<S, B> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggerMiddleware { service })
    }
}

pub struct LoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Utc::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let elapsed = Utc::now().signed_duration_since(start_time).num_milliseconds();

            if res.status().is_server_error() {
                error!(
                    target: "my_actix_api",
                    "Error occurred - Timestamp: {}, Method: {}, Path: {}, Status: {}, Elapsed: {}ms",
                    start_time, method, path, res.status(), elapsed
                );
            } else {
                println!(
                    "Request processed - Timestamp: {}, Method: {}, Path: {}, Status: {}, Elapsed: {}ms",
                    start_time, method, path, res.status(), elapsed
                );
            }

            Ok(res)
        })
    }
}