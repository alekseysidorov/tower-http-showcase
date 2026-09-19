use std::future::Future;

use showcase_api::model::{HelloRequest, HelloResponse};
use tower::{BoxError, Service, ServiceBuilder};
use tower_http_client::{ResponseExt as _, ServiceExt as _, rewrite_uri::RewriteUriLayer};

/// An HTTP client service erased behind a cloneable Tower service.
pub type BoxedHttpClient = tower::util::BoxCloneSyncService<
    http::Request<reqwest::Body>,
    http::Response<reqwest::Body>,
    BoxError,
>;

/// Client-side API implemented for compatible Tower HTTP services.
pub trait HelloClient {
    /// Send a hello request and decode the response.
    fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> impl Future<Output = Result<HelloResponse, BoxError>> + Send;
}

impl<S> HelloClient for S
where
    S: Service<
            http::Request<reqwest::Body>,
            Response = http::Response<reqwest::Body>,
            Error = BoxError,
        > + Send,
    S::Future: Send + 'static,
{
    #[expect(
        clippy::manual_async_fn,
        reason = "The trait explicitly guarantees that this future is Send."
    )]
    fn say_hello(
        &mut self,
        request: HelloRequest,
    ) -> impl Future<Output = Result<HelloResponse, BoxError>> + Send {
        async move {
            let response = self.get("/hello").json(&request)?.send().await?;
            let body = response.body_reader().json::<HelloResponse>().await?;

            Ok(body)
        }
    }
}

/// Add URI rewriting to an HTTP service without erasing its concrete type.
///
/// The origin must be absolute and must not contain a query. Any path prefix
/// in the origin is preserved when relative request paths are appended.
pub fn with_origin<S>(
    service: S,
    origin: http::Uri,
) -> Result<
    impl Service<
        http::Request<reqwest::Body>,
        Response = http::Response<reqwest::Body>,
        Error = BoxError,
        Future: Send,
    > + Clone
    + Send
    + Sync
    + 'static,
    BoxError,
>
where
    S: Service<http::Request<reqwest::Body>, Response = http::Response<reqwest::Body>>
        + Clone
        + Send
        + Sync
        + 'static,
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

    Ok(service)
}
