use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::{
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::timeout;

use crate::models::error::AppError;

pub struct TimeoutHandler {
    pub duration: Duration,
}

impl TimeoutHandler {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TimeoutHandler
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TimeoutHandleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TimeoutHandleMiddleware {
            service: Rc::new(service),
            duration: self.duration,
        })
    }
}

pub struct TimeoutHandleMiddleware<S> {
    service: Rc<S>,
    duration: Duration,
}

impl<S, B> Service<ServiceRequest> for TimeoutHandleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let duration = self.duration;
        Box::pin(async move {
            match timeout(duration, srv.call(req)).await {
                Ok(result) => result,
                Err(_) => Err(Error::from(AppError::Timeout)),
            }
        })
    }
}
