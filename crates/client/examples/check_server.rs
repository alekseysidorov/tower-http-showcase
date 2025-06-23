use std::time::Duration;

use context_logger::ContextLogger;
use fastrace::collector::{Config, ConsoleReporter};
use fastrace_jaeger::JaegerReporter;
use fastrace_tower::FastraceClientLayer;
use futures_util::StreamExt as _;
use http::{HeaderValue, header::USER_AGENT};
use http_body_util::BodyExt as _;
use log::{LevelFilter, error, info};
use showcase_api::{HelloService, NODES_COUNT, model::HelloRequest};
use showcase_client::{BoxedHttpClient, HelloClient};
use structured_logger::{Builder, async_json::new_writer};
use tower::{
    BoxError, ServiceBuilder,
    balance::p2c::Balance,
    load::{CompleteOnResponse, PeakEwma},
};
use tower_http::ServiceBuilderExt as _;
use tower_http_client::adapters::reqwest::{HttpClientLayer, into_reqwest_body};

fn make_tower_http_client(client: reqwest::Client, node_address: String) -> BoxedHttpClient {
    ServiceBuilder::new()
        .layer_fn(BoxedHttpClient::new)
        .layer(FastraceClientLayer)
        // Add some layers.
        .map_request(move |mut request: http::Request<_>| {
            // Add node address to the request URI, since the underlying client relies on it.
            *request.uri_mut() = [&node_address, request.uri().path()]
                .concat()
                .parse()
                .unwrap();
            info!(node_address; "Sending request to node");
            request
        })
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        // Make client compatible with the `tower-http` layers.
        .map_err(BoxError::from)
        .map_response_body(|body: reqwest::Body| body.map_err(BoxError::from).boxed())
        .map_request_body(into_reqwest_body)
        .layer(HttpClientLayer)
        .service(client)
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let level = LevelFilter::Info;
    ContextLogger::new(
        Builder::with_level(level.as_str())
            .with_target_writer("*", new_writer(tokio::io::stdout()))
            .build(),
    )
    .init(level);

    // Initialize fastrace reporter.
    let reporter = JaegerReporter::new("127.0.0.1:6831".parse()?, "tower-http-showcase-client")?;
    fastrace::set_reporter(reporter, Config::default());

    let server_address = format!("http://localhost:{}", showcase_api::DEFAULT_SERVER_PORT);

    let inner_client = ServiceBuilder::new()
        .buffer(256)
        .concurrency_limit(16)
        .service(Balance::new(tower::discover::ServiceList::new(
            (0..NODES_COUNT).map(move |node_id| {
                let node_address = format!("{server_address}/node/{node_id}");
                let inner = make_tower_http_client(reqwest::Client::new(), node_address.clone());
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
                let hello_client = hello_client.clone();
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

    fastrace::flush();
    Ok(())
}
