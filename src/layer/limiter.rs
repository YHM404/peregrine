use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use futures::{ready, FutureExt};
use tokio::sync::Mutex;
use tower::{
    limit::{rate::Rate, RateLimit},
    Service,
};

#[derive(Clone)]
struct Limiter<T> {
    limit: Arc<Mutex<RateLimit<T>>>,
}

impl<T> Limiter<T> {
    pub fn new(inner: T, qps: u64) -> Self {
        let limit = RateLimit::new(inner, Rate::new(qps, Duration::from_secs(1)));
        Self {
            limit: Arc::new(Mutex::new(limit)),
        }
    }
}

impl<T, Request> Service<Request> for Limiter<T>
where
    T: Service<Request> + Send + 'static,
    T::Future: Send,
    T: Clone,
    Request: Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let mut lock = Box::pin(self.limit.lock());
        let mut lock = ready!(lock.poll_unpin(cx));
        lock.get_mut().poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let this = self.clone();
        async move {
            let mut guard = this.limit.lock().await;
            guard.get_mut().call(req).await
        }
        .boxed()
    }
}
