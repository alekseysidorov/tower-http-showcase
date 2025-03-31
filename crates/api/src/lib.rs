use std::future::Future;

pub mod model;

pub const DEFAULT_SERVER_PORT: u16 = 8080;

pub const NODES_COUNT: u32 = 16;

pub trait HelloService {
    type TransportError;

    fn say_hello(
        &mut self,
        request: model::HelloRequest,
    ) -> impl Future<Output = Result<model::HelloResponse, Self::TransportError>>;
}
