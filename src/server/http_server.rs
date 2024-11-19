use anyhow::Result;
use hyper::{server::conn::http2, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{error, info};
use tokio::net::TcpListener;
use tower::ServiceExt;

use crate::{config, layer::balance};

pub(crate) struct HttpServer {
    config: config::ServerConfig,
}

impl HttpServer {
    pub(crate) fn new(config: config::ServerConfig) -> Self {
        Self { config }
    }

    pub(crate) async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.config.port)).await?;
        info!("Listening on {}", listener.local_addr()?);

        loop {
            if let Ok((stream, _addr)) = listener.accept().await {
                let io = TokioIo::new(stream);
                let svc = balance::Endpoints::new(self.config.backends.values())?;
                let handler = service_fn(move |req| svc.clone().oneshot(req));
                tokio::task::spawn(async move {
                    if let Err(err) = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                        .serve_connection(io, handler)
                        .await
                    {
                        error!("Failed to serve connection: {:?}", err);
                    }
                });
            }
        }
    }
}
