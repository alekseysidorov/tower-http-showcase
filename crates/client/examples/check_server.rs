use std::time::Duration;

use futures_util::StreamExt as _;
use http::{HeaderValue, header::USER_AGENT};
use log::{error, info};
use showcase_api::{HelloService, NODES_COUNT, model::HelloRequest};
use showcase_client::{BoxedHttpClient, HelloClient};
use structured_logger::{Builder, async_json::new_writer};
use tower::{
    BoxError, ServiceBuilder, ServiceExt as _,
    balance::p2c::Balance,
    load::{CompleteOnResponse, PeakEwma},
};
use tower_http::ServiceBuilderExt as _;
use tower_http_client::rewrite_uri::RewriteUriLayer;
use tower_reqwest::HttpClientLayer;

fn make_client(client: reqwest::Client, node_address: String) -> BoxedHttpClient {
    let log_node_address = node_address.clone();
    let service = ServiceBuilder::new()
        // Resolve relative client URIs against this node's base URI.
        .layer(RewriteUriLayer::new(move |uri: &http::Uri| {
            let path_and_query = uri.path_and_query().map_or("/", |value| value.as_str());
            format!("{node_address}{path_and_query}")
                .parse::<http::Uri>()
                .map_err(BoxError::from)
        }))
        .map_err(BoxError::from)
        .map_request(move |request: http::Request<_>| {
            info!(node_address: log_node_address; "Sending request to node");
            request
        })
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        // Make client compatible with the `tower-http` layers.
        .layer(HttpClientLayer)
        .service(client)
        .map_err(eyre::Error::from);
    tower::util::BoxCloneSyncService::new(service)
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let server_address = format!("http://localhost:{}", showcase_api::DEFAULT_SERVER_PORT);
    let inner_client = ServiceBuilder::new()
        .buffer(256)
        .concurrency_limit(16)
        .service(Balance::new(tower::discover::ServiceList::new(
            (0..NODES_COUNT).map(move |node_id| {
                let node_address = format!("{server_address}/node/{node_id}");
                let inner = make_client(reqwest::Client::new(), node_address.clone());
                PeakEwma::new(
                    inner,
                    Duration::from_millis(10),
                    // 1s
                    1_000_000_000f64,
                    CompleteOnResponse::default(),
                )
            }),
        )));

    let hello_client = HelloClient::new(inner_client);
    futures_util::stream::iter(0..2048)
        .map({
            let hello_client = hello_client.clone();
            move |_| {
                let mut hello_client = hello_client.clone();
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
