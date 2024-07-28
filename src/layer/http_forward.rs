use std::{
    str::FromStr,
    task::{Context, Poll},
};

use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use hyper::{body::Body, Request, Response, Uri};
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use tower::Service;

use crate::config::BackendConfig;

#[derive(Debug, Clone)]
pub(crate) struct Forward<B> {
    uri: Uri,
    client: Client<HttpConnector, B>,
}

impl<B> Forward<B>
where
    B: Body + Send + 'static,
    <B as Body>::Data: Send,
{
    pub(crate) fn new(config: &BackendConfig) -> Result<Self> {
        let uri = Uri::from_str(format!("http://{}:{}", config.host, config.port).as_str())?;
        Ok(Self {
            uri,
            client: Client::builder(hyper_util::rt::TokioExecutor::new())
                .http2_only(config.enable_h2c)
                .build_http(),
        })
    }
}

impl<B> Service<Request<B>> for Forward<B>
where
    B: Body + Send + 'static + Unpin,
    B::Data: Send,
    <B as Body>::Error: Into<Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>>,
{
    type Response = Response<hyper::body::Incoming>;

    type Error = anyhow::Error;

    type Future = BoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        // replace the request URI with the target URI
        req.uri_mut().clone_from(&self.uri);

        self.client
            .call(req)
            .map(|res| res.map_err(Into::into))
            .boxed()
    }
}
