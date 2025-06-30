use std::{future::Future, sync::Arc};

use fastrace::local::LocalSpan;
use tokio::sync::Mutex;

use crate::delay_iter::DelayIter;

#[derive(Debug)]
pub struct AppState {
    node_id: u32,
    delays_iter: Mutex<DelayIter>,
}

pub type SharedAppState = Arc<AppState>;

// TODO When example is ready, move this implementation to the more appropriate place.
pub trait HelloService {
    fn say_hello(&self, name: &str) -> impl Future<Output = String> + Send + Sync;
}

#[derive(Debug)]
struct HelloServiceImpl<'a> {
    node_id: u32,
    delays_iter: &'a Mutex<DelayIter>,
}

impl HelloService for HelloServiceImpl<'_> {
    #[fastrace::trace(short_name = true)]
    async fn say_hello(&self, name: &str) -> String {
        LocalSpan::add_properties(|| {
            [
                ("node.id", self.node_id.to_string()),
                ("fastrace", "rocks!".to_string()),
            ]
        });

        let duration = self.delays_iter.lock().await.next().unwrap();
        tokio::time::sleep(duration).await;

        format!("Hello, {}!", name)
    }
}

impl AppState {
    pub fn new(node_id: u32, delay_iter: DelayIter) -> Self {
        Self {
            node_id,
            delays_iter: Mutex::new(delay_iter),
        }
    }

    pub fn hello_service(&self) -> impl HelloService + '_ {
        HelloServiceImpl {
            node_id: self.node_id,
            delays_iter: &self.delays_iter,
        }
    }
}
