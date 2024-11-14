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

pub(crate) struct Endpoints<D, B>
where
    D: Discover + Unpin,
    D::Key: Hash + Clone,
{
    backends_config: Vec<BackendConfig>,
    pool: Balance<D, Request<B>>,
}

impl<B> Endpoints<ServiceList<Vec<Constant<Forward<B>, i32>>>, B>
where
    B: Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub(crate) fn new<'a>(
        backends_config: impl Iterator<Item = &'a BackendConfig>,
    ) -> Result<Self> {
        let backends_config: Vec<_> = backends_config.cloned().collect();
        let server_list: Vec<_> = backends_config
            .iter()
            .map(|backend| Forward::new(backend).map(|fwd| Constant::new(fwd, 1)))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            backends_config,
            pool: Balance::new(ServiceList::new(server_list)),
        })
    }
}

impl<B> Clone for Endpoints<ServiceList<Vec<Constant<Forward<B>, i32>>>, B>
where
    B: Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fn clone(&self) -> Self {
        // expect is ok here because we already checked the config in the new function
        Self::new(self.backends_config.iter()).expect("Failed to clone Backends")
    }
}

impl<B1, B2, D> Service<Request<B1>> for Endpoints<D, B1>
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

    fn call(&mut self, req: Request<B1>) -> Self::Future {
        self.pool
            .call(req)
            .map_err(|err| anyhow!(err.to_string()))
            .boxed()
    }
}
