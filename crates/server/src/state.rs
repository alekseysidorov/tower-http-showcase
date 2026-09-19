use std::{future::Future, sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::delay_iter::DelayIter;

#[derive(Debug)]
pub struct AppState {
    delays_iter: Mutex<DelayIter>,
    worker_id: String,
    worker_delay: Duration,
}

pub type SharedAppState = Arc<AppState>;

// TODO When example is ready, move this implementation to the more appropriate place.
pub trait HelloService {
    fn say_hello(&self, name: &str) -> impl Future<Output = String> + Send + Sync;
}

#[derive(Debug)]
struct HelloServiceImpl<'a> {
    delays_iter: &'a Mutex<DelayIter>,
}

impl HelloService for HelloServiceImpl<'_> {
    async fn say_hello(&self, name: &str) -> String {
        let duration = self.delays_iter.lock().await.next().unwrap();
        tokio::time::sleep(duration).await;

        format!("Hello, {}!", name)
    }
}

impl AppState {
    pub fn new(delay_iter: DelayIter) -> Self {
        Self::with_worker(delay_iter, "tokio-worker-1", Duration::ZERO)
    }

    pub fn with_worker(
        delay_iter: DelayIter,
        worker_id: impl Into<String>,
        worker_delay: Duration,
    ) -> Self {
        Self {
            delays_iter: Mutex::new(delay_iter),
            worker_id: worker_id.into(),
            worker_delay,
        }
    }

    pub fn hello_service(&self) -> impl HelloService + '_ {
        HelloServiceImpl {
            delays_iter: &self.delays_iter,
        }
    }

    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }

    pub fn worker_delay(&self) -> Duration {
        self.worker_delay
    }
}
