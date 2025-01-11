use std::future::Future;

use crate::state::SharedAppState;

pub trait HelloService {
    fn say_hello(&self, name: String) -> impl Future<Output = String>;
}

impl HelloService for SharedAppState {
    async fn say_hello(&self, name: String) -> String {
        format!("Hello, {}!", name)
    }
}
