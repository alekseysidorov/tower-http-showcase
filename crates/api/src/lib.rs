use std::future::Future;

pub mod model;

/// Separates failures to execute a request from failures reported by the API.
#[derive(Debug)]
pub enum HelloError<T> {
    Transport(T),
    Api(http::StatusCode),
    Protocol(String),
}

pub const DEFAULT_SERVER_PORT: u16 = 8080;

pub const NODES_COUNT: u32 = 16;

pub trait HelloService {
    type TransportError;

    fn say_hello(
        &mut self,
        request: model::HelloRequest,
    ) -> impl Future<Output = Result<model::HelloResponse, HelloError<Self::TransportError>>> + Send;
}
