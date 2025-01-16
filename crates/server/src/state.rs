use std::{future::Future, sync::Arc};

use tokio::sync::Mutex;

use crate::delay_iter::DelayIter;

#[derive(Debug)]
pub struct AppState {
    delays_iter: Mutex<DelayIter>,
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
        Self {
            delays_iter: Mutex::new(delay_iter),
        }
    }

    pub fn hello_service(&self) -> impl HelloService + '_ {
        HelloServiceImpl {
            delays_iter: &self.delays_iter,
        }
    }
}
