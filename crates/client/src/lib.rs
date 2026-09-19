use showcase_api::{
    HelloService,
    model::{HelloRequest, HelloResponse},
};
use tower::{BoxError, Service, ServiceBuilder};
use tower_http_client::{ResponseExt as _, ServiceExt as _, rewrite_uri::RewriteUriLayer};

type HttpRequest = http::Request<reqwest::Body>;
type HttpResponse = http::Response<reqwest::Body>;
type InnerHttpClient = tower::util::BoxCloneSyncService<HttpRequest, HttpResponse, BoxError>;

/// Implementation agnostic HTTP client.
#[derive(Clone)]
pub struct BoxedHttpClient(InnerHttpClient);

impl BoxedHttpClient {
    /// Erase the concrete type of an HTTP service.
    pub fn from_service<S>(service: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = BoxError>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        Self(tower::util::BoxCloneSyncService::new(service))
    }

    /// Rewrite relative request URIs to the given absolute origin before sending.
    pub fn with_origin<S>(service: S, origin: http::Uri) -> Result<Self, BoxError>
    where
        S: Service<HttpRequest, Response = HttpResponse> + Clone + Send + Sync + 'static,
        S::Error: std::error::Error + Send + Sync + 'static,
        S::Future: Send + 'static,
    {
        if origin.scheme().is_none() || origin.authority().is_none() || origin.query().is_some() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "origin must be an absolute URI without a query",
            )
            .into());
        }

        let origin = origin.to_string().trim_end_matches('/').to_owned();
        let service = ServiceBuilder::new()
            .layer(RewriteUriLayer::new(move |uri: &http::Uri| {
                let path_and_query = uri.path_and_query().map_or("/", |value| value.as_str());
                format!("{origin}{path_and_query}")
                    .parse::<http::Uri>()
                    .map_err(BoxError::from)
            }))
            .map_err(BoxError::from)
            .service(service);

        Ok(Self::from_service(service))
    }
}

impl Service<HttpRequest> for BoxedHttpClient {
    type Response = HttpResponse;
    type Error = BoxError;
    type Future = <InnerHttpClient as Service<HttpRequest>>::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        self.0.call(request)
    }
}

impl HelloService for BoxedHttpClient {
    type TransportError = BoxError;

    async fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> Result<HelloResponse, Self::TransportError> {
        let response = self.0.get("/hello").json(&request)?.send().await?;
        let body = response.body_reader().json::<HelloResponse>().await?;

        Ok(body)
    }
}
