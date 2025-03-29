use showcase_api::{
    model::{HelloRequest, HelloResponse},
    HelloService,
};
use tower_http_client::{ResponseExt as _, ServiceExt as _};

/// Implementation agnostic HTTP client.
type BoxedClient = tower::util::BoxCloneSyncService<
    http::Request<reqwest::Body>,
    http::Response<reqwest::Body>,
    eyre::Error,
>;

#[derive(Clone, Debug)]
pub struct HelloClient(BoxedClient);

impl HelloClient {
    pub fn new(inner: impl Into<BoxedClient>) -> Self {
        Self(inner.into())
    }
}

impl HelloService for HelloClient {
    type TransportError = eyre::Error;

    async fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> Result<HelloResponse, Self::TransportError> {
        let response = self.0.get("/hello").json(&request)?.send().await?;
        let body = response.body_reader().json::<HelloResponse>().await?;

        Ok(body)
    }
}
