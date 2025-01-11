use showcase_api::{
    model::{HelloRequest, HelloResponse},
    HelloServiceClient,
};
use tower_http_client::{ResponseExt as _, ServiceExt as _};

/// Implementation agnostic HTTP client.
pub type HttpClient = tower_http_client::util::BoxCloneSyncService<
    http::Request<reqwest::Body>,
    http::Response<reqwest::Body>,
    eyre::Error,
>;

#[derive(Clone, Debug)]
pub struct HttpClientWithUrl {
    base_url: String,
    client: HttpClient,
}

impl HttpClientWithUrl {
    pub fn new(base_url: impl Into<String>, client: HttpClient) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

impl HelloServiceClient for HttpClientWithUrl {
    type TransportError = eyre::Error;

    async fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> Result<HelloResponse, Self::TransportError> {
        let response = self
            .client
            .get(format!("{}/hello", self.base_url))
            .json(&request)?
            .send()?
            .await?;
        let body = response.body_reader().json::<HelloResponse>().await?;

        Ok(body)
    }
}
