use bytes::Bytes;
use http_body::Body;
use http_body_util::combinators::BoxBody;
use showcase_api::{
    HelloService,
    model::{HelloRequest, HelloResponse},
};
use tower::{BoxError, Service, ServiceExt as _, util::BoxCloneSyncService};
use tower_http_client::{ResponseExt as _, ServiceExt as _};

/// A body that can be cloned in order to be sent multiple times.
pub type CloneableBody = http_body_util::Full<Bytes>;
/// A type-erased HTTP client that is completely implementation-agnostic.
pub type BoxedHttpClient = BoxCloneSyncService<
    http::Request<CloneableBody>,
    http::Response<BoxBody<Bytes, BoxError>>,
    BoxError,
>;

#[derive(Clone, Debug)]
pub struct HelloClient<S>(S);

impl<S> HelloClient<S> {
    pub fn new(inner: S) -> Self {
        Self(inner)
    }
}

impl<S, RespBody> HelloService for HelloClient<S>
where
    S: Service<http::Request<CloneableBody>, Response = http::Response<RespBody>, Error = BoxError>
        + Clone
        + Send
        + 'static,
    RespBody: Body<Error = BoxError> + Send + 'static,
    RespBody::Data: bytes::Buf + Send,
    S::Future: Send,
{
    type TransportError = S::Error;

    async fn say_hello(
        &self,
        request: HelloRequest,
    ) -> Result<HelloResponse, Self::TransportError> {
        let mut service = self.0.clone();
        service.ready().await?;

        let response = service.get("/hello").json(&request)?.send().await?;
        let body = response.body_reader().json::<HelloResponse>().await?;

        Ok(body)
    }
}
