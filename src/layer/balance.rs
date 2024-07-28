use std::{fmt::Debug, hash::Hash};

use crate::{config::BackendConfig, layer::http_forward::Forward};
use anyhow::{anyhow, Result};
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use http::{Request, Response};
use hyper::body::Body;
use tower::{
    balance::p2c::Balance,
    discover::{Discover, ServiceList},
    load::{Constant, Load},
    Service,
};

pub(crate) struct Backends<D, B>
where
    D: Discover + Unpin,
    <D as Discover>::Key: Hash + Clone,
{
    backends_config: Vec<BackendConfig>,
    pool: Balance<D, Request<B>>,
}

impl<B> Backends<ServiceList<Vec<Constant<Forward<B>, i32>>>, B>
where
    B: Body + Send + Unpin + 'static,
    <B as Body>::Data: Send,
    <B as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub(crate) fn new<'a>(
        backends_config: impl Iterator<Item = &'a BackendConfig>,
    ) -> Result<Self> {
        let backends_config: Vec<_> = backends_config.cloned().collect();
        let mut server_list = Vec::with_capacity(backends_config.len());
        for backend in &backends_config {
            let service = Constant::new(Forward::new(backend)?, 1);
            server_list.push(service);
        }

        Ok(Self {
            backends_config,
            pool: Balance::new(ServiceList::new(server_list)),
        })
    }
}

impl<B> Clone for Backends<ServiceList<Vec<Constant<Forward<B>, i32>>>, B>
where
    B: Body + Send + Unpin + 'static,
    <B as Body>::Data: Send,
    <B as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fn clone(&self) -> Self {
        match Self::new(self.backends_config.iter()) {
            Ok(pool) => pool,
            Err(err) => panic!("Failed to clone Backends: {}", err),
        }
    }
}

impl<B1, B2, D> Service<Request<B1>> for Backends<D, B1>
where
    D: Discover + Unpin,
    D::Key: Hash + Clone,
    D::Service: Service<Request<B1>, Response = Response<B2>> + Load,
    <D::Service as Service<Request<B1>>>::Error:
        Into<Box<(dyn std::error::Error + Send + Sync)>> + 'static,
    <D::Service as Service<Request<B1>>>::Future: Send + 'static,
    <D::Service as Load>::Metric: Debug,
    D::Error: Into<Box<(dyn std::error::Error + Send + Sync)>>,
    B2: Body,
{
    type Response = Response<B2>;
    type Error = anyhow::Error;

    type Future = BoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.pool
            .poll_ready(cx)
            .map_err(|err| anyhow!(err.to_string()))
    }

    fn call(&mut self, req: http::Request<B1>) -> Self::Future {
        self.pool
            .call(req)
            .map_err(|err| anyhow!(err.to_string()))
            .boxed()
    }
}
