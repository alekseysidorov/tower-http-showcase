use std::future::Future;

pub mod model;

pub const DEFAULT_SERVER_PORT: u16 = 8080;

pub trait HelloServiceClient {
    type TransportError;

    fn say_hello(
        &mut self,
        request: model::HelloRequest,
    ) -> impl Future<Output = Result<model::HelloResponse, Self::TransportError>>;
}
