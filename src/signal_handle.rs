use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinHandle;

pub(crate) struct SignalHandler {
    handles: Vec<JoinHandle<()>>,
}

impl SignalHandler {
    pub(crate) fn new() -> Self {
        Self { handles: vec![] }
    }

    pub(crate) fn handle_signal(
        &mut self,
        kind: SignalKind,
        handler: impl Fn() + Send + Sync + 'static,
    ) -> anyhow::Result<&mut Self> {
        let mut sig = signal(kind)?;
        let handle = tokio::spawn(async move {
            while let Some(_) = sig.recv().await {
                handler();
            }
        });
        self.handles.push(handle);
        Ok(self)
    }

    pub(crate) async fn run(self) -> anyhow::Result<()> {
        futures::future::join_all(self.handles).await;
        Ok(())
    }
}
