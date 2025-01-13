use std::{future::Future, sync::Arc};

use tokio::sync::Mutex;

use crate::{config::AppConfig, delay_iter::DelayIter};

#[derive(Debug)]
pub struct AppState {
    nodes: Vec<Mutex<DelayIter>>,
}

pub type SharedAppState = Arc<AppState>;

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let nodes = (0..config.nodes_count)
            .map(|node_id| {
                Mutex::new(DelayIter::new(
                    node_id,
                    config.response_delays.min..config.response_delays.max,
                ))
            })
            .collect();

        Self { nodes }
    }
}

// TODO When example is ready, move this implementation to the more appropriate place.
pub trait HelloService {
    fn say_hello(&self, name: &str) -> impl Future<Output = String> + Send + Sync;
}

#[derive(Debug)]
struct HelloServiceImpl<'a> {
    node: &'a Mutex<DelayIter>,
}

impl HelloService for HelloServiceImpl<'_> {
    async fn say_hello(&self, name: &str) -> String {
        let duration = self.node.lock().await.next().unwrap();
        tokio::time::sleep(duration).await;

        format!("Hello, {}!", name)
    }
}

impl AppState {
    pub fn hello_service<'a>(&'a self, node_id: u16) -> Option<impl HelloService + 'a> {
        self.nodes
            .get(node_id as usize)
            .map(|node| HelloServiceImpl { node })
    }
}
