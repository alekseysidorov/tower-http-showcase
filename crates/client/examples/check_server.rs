use std::time::Duration;

use futures_util::StreamExt as _;
use http::{HeaderValue, Uri, header::USER_AGENT};
use log::{error, info};
use showcase_api::{HelloService, NODES_COUNT, model::HelloRequest};
use showcase_client::BoxedHttpClient;
use structured_logger::{Builder, async_json::new_writer};
use tower::{
    BoxError, ServiceBuilder,
    balance::p2c::Balance,
    load::{CompleteOnResponse, PeakEwma},
};
use tower_http::ServiceBuilderExt as _;
use tower_reqwest::HttpClientLayer;

fn make_client(client: reqwest::Client, node_address: Uri) -> Result<BoxedHttpClient, BoxError> {
    let log_node_address = node_address.to_string();
    let service = ServiceBuilder::new()
        .map_request(move |request: http::Request<_>| {
            info!(node_address = log_node_address; "Sending request to node");
            request
        })
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        // Make client compatible with the `tower-http` layers.
        .layer(HttpClientLayer)
        .service(client);
    BoxedHttpClient::with_origin(service, node_address)
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

    let hello_client = BoxedHttpClient::from_service(inner_client);
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
