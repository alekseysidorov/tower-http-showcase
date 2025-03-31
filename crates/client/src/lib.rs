use showcase_api::{
    model::{HelloRequest, HelloResponse},
    HelloService,
};
use tower::{BoxError, Service};
use tower_http_client::{ResponseExt as _, ServiceExt as _};

/// Implementation agnostic HTTP client.
pub type BoxedHttpClient = tower::util::BoxCloneSyncService<
    http::Request<reqwest::Body>,
    http::Response<reqwest::Body>,
    eyre::Error,
>;

#[derive(Clone, Debug)]
pub struct HelloClient<S>(S);

impl<S> HelloClient<S> {
    pub fn new(inner: S) -> Self {
        Self(inner)
    }
}

impl<S> HelloService for HelloClient<S>
where
    S: Service<
        http::Request<reqwest::Body>,
        Response = http::Response<reqwest::Body>,
        Error = BoxError,
    >,
    S::Future: Send + 'static,
{
    type TransportError = S::Error;

    async fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> Result<HelloResponse, Self::TransportError> {
        let response = self.0.get("/hello").json(&request)?.send().await?;
        let body = response.body_reader().json::<HelloResponse>().await?;

        Ok(body)
    }
}
