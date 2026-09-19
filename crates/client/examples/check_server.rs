use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt as _;
use http::{HeaderValue, Request, Response, Uri, header::USER_AGENT};
use http_body_util::combinators::BoxBody;
use log::{error, info};
use showcase_api::{HelloService as _, NODES_COUNT, model::HelloRequest};
use showcase_client::{
    BoxedHttpClient, CloneableBody, HelloClient, into_tower_http_client, with_origin,
};
use structured_logger::{Builder, async_json::new_writer};
use tower::{
    BoxError, Service, ServiceBuilder,
    balance::p2c::Balance,
    load::{CompleteOnResponse, PeakEwma},
    retry::Policy,
    util::BoxCloneSyncService,
};
use tower_http::ServiceBuilderExt as _;

#[derive(Clone)]
struct RetryServerErrors {
    remaining: u8,
}

impl<B, R, E> Policy<Request<B>, Response<R>, E> for RetryServerErrors
where
    B: Clone,
{
    type Future = tokio::time::Sleep;

    fn retry(
        &mut self,
        _request: &mut Request<B>,
        result: &mut Result<Response<R>, E>,
    ) -> Option<Self::Future> {
        let retryable = result
            .as_ref()
            .map_or(true, |response| response.status().is_server_error());
        if !retryable || self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        Some(tokio::time::sleep(Duration::from_millis(25)))
    }

    fn clone_request(&mut self, request: &Request<B>) -> Option<Request<B>> {
        Some(request.clone())
    }
}

fn make_client(
    client: reqwest::Client,
    node_address: Uri,
) -> Result<
    impl Service<
        Request<CloneableBody>,
        Response = Response<BoxBody<Bytes, BoxError>>,
        Error = BoxError,
        Future: Send,
    > + Clone
    + Send
    + Sync
    + 'static,
    BoxError,
> {
    let log_node_address = node_address.to_string();
    let service = into_tower_http_client(client);
    let service = with_origin(service, node_address)?;
    Ok(ServiceBuilder::new()
        .retry(RetryServerErrors { remaining: 2 })
        .map_request(move |request: http::Request<_>| {
            info!(node_address = log_node_address; "Sending request to node");
            request
        })
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        .service(service))
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let server_address = format!("http://localhost:{}", showcase_api::DEFAULT_SERVER_PORT);
    let nodes = (0..NODES_COUNT)
        .map(|node_id| {
            let node_address: Uri = format!("{server_address}/node/{node_id}").parse()?;
            let inner = make_client(reqwest::Client::new(), node_address.clone())?;
            Ok(PeakEwma::new(
                inner,
                Duration::from_millis(10),
                // 1s
                1_000_000_000f64,
                CompleteOnResponse::default(),
            ))
        })
        .collect::<Result<Vec<_>, BoxError>>()?;
    let inner_client = ServiceBuilder::new()
        .buffer(256)
        .concurrency_limit(16)
        .service(Balance::new(tower::discover::ServiceList::new(nodes)));

    // Erase the fully composed stack only at the public homogeneous client boundary.
    let hello_client: BoxedHttpClient = BoxCloneSyncService::new(inner_client);
    futures_util::stream::iter(0..2048)
        .map({
            let hello_client = hello_client.clone();
            move |_| {
                let mut hello_client = HelloClient::new(hello_client.clone());
                async move {
                    hello_client
                        .say_hello(HelloRequest {
                            name: "Alice".to_string(),
                        })
                        .await
                }
            }
        })
        .for_each_concurrent(16, async |result| match result.await {
            Ok(response) => {
                info!(
                    response:serde;
                    "Received response"
                );
            }
            Err(err) => {
                error!(
                    err:?;
                    "Failed to receive response"
                );
            }
        })
        .await;

    Ok(())
}
