use bytes::Bytes;
use http::{Request, Response, Uri};
use http_body_util::{BodyExt as _, Full, combinators::BoxBody};
use showcase_api::{
    HelloError, HelloService,
    model::{HelloRequest, HelloResponse},
};
use tower::{BoxError, Service, ServiceBuilder};
use tower_http::ServiceBuilderExt as _;
use tower_http_client::{
    ResponseExt as _, ServiceExt as _,
    rewrite_uri::{RewriteUri, RewriteUriLayer},
};
use tower_reqwest::HttpClientLayer;

/// A request body that middleware can clone and replay (for example, on retry).
pub type CloneableBody = Full<Bytes>;

/// A normalized HTTP service boundary around the concrete reqwest transport.
pub type BoxedHttpClient = tower::util::BoxCloneSyncService<
    Request<CloneableBody>,
    Response<BoxBody<Bytes, BoxError>>,
    BoxError,
>;

/// Adapt reqwest's concrete request/response bodies to the composable Tower contract.
pub fn into_tower_http_client(
    client: reqwest::Client,
) -> impl Send
+ Clone
+ Service<
    Request<CloneableBody>,
    Response = Response<BoxBody<Bytes, BoxError>>,
    Error = BoxError,
    Future: Send,
> {
    ServiceBuilder::new()
        .map_err(BoxError::from)
        .map_response_body(|body: reqwest::Body| body.map_err(BoxError::from).boxed())
        .map_request_body(reqwest::Body::wrap)
        .layer(HttpClientLayer)
        .service(client)
}

/// Add node-specific URI rewriting while preserving request path and query.
pub fn with_origin<S>(
    service: S,
    origin: Uri,
) -> Result<
    impl Clone
    + Send
    + Sync
    + Service<
        Request<CloneableBody>,
        Response = Response<BoxBody<Bytes, BoxError>>,
        Error = BoxError,
        Future: Send,
    >,
    BoxError,
>
where
    S: Clone
        + Send
        + Sync
        + Service<
            Request<CloneableBody>,
            Response = Response<BoxBody<Bytes, BoxError>>,
            Error = BoxError,
        >,
    S::Future: Send + 'static,
{
    let rewriter = BaseUri::from_uri(origin)?;
    Ok(ServiceBuilder::new()
        .layer(RewriteUriLayer::new(rewriter))
        .map_err(BoxError::from)
        .service(service))
}

#[derive(Clone)]
struct BaseUri {
    scheme: http::uri::Scheme,
    authority: http::uri::Authority,
    path_prefix: String,
}

impl BaseUri {
    fn from_uri(uri: Uri) -> Result<Self, BoxError> {
        let parts = uri.into_parts();
        let scheme = parts
            .scheme
            .ok_or_else(|| invalid_origin("missing scheme"))?;
        let authority = parts
            .authority
            .ok_or_else(|| invalid_origin("missing authority"))?;
        if parts
            .path_and_query
            .as_ref()
            .is_some_and(|pq| pq.query().is_some())
        {
            return Err(invalid_origin("origin must not contain a query"));
        }
        let path_prefix = parts
            .path_and_query
            .map_or_else(String::new, |pq| pq.path().trim_end_matches('/').to_owned());
        Ok(Self {
            scheme,
            authority,
            path_prefix,
        })
    }
}

impl RewriteUri for BaseUri {
    type Error = http::Error;

    fn rewrite_uri(&mut self, uri: &Uri) -> Result<Uri, Self::Error> {
        let path_and_query = uri.path_and_query().map_or("/", |pq| pq.as_str());
        let (path, query) = path_and_query
            .split_once('?')
            .unwrap_or((path_and_query, ""));
        let full_path = format!("{}{path}", self.path_prefix);
        let path_and_query = if query.is_empty() {
            full_path
        } else {
            format!("{full_path}?{query}")
        };
        Uri::builder()
            .scheme(self.scheme.clone())
            .authority(self.authority.clone())
            .path_and_query(path_and_query)
            .build()
    }
}

fn invalid_origin(message: &'static str) -> BoxError {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into()
}

/// Typed domain client over any normalized Tower HTTP service.
pub struct HelloClient<S> {
    service: S,
}

impl<S> HelloClient<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> HelloService for HelloClient<S>
where
    S: Service<Request<CloneableBody>, Response = Response<BoxBody<Bytes, BoxError>>> + Send,
    S::Error: Send + 'static,
    S::Future: Send + 'static,
{
    type TransportError = S::Error;

    async fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> Result<HelloResponse, HelloError<S::Error>> {
        let response = self
            .service
            .get("/hello")
            .json(&request)
            .map_err(|error| HelloError::Protocol(error.to_string()))?
            .send()
            .await
            .map_err(HelloError::Transport)?;

        let status = response.status();
        if !status.is_success() {
            return Err(HelloError::Api(status));
        }

        response
            .body_reader()
            .json::<HelloResponse>()
            .await
            .map_err(|error| HelloError::Protocol(error.to_string()))
    }
}
