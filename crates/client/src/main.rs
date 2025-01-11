use http::{header::USER_AGENT, HeaderValue};
use log::info;
use showcase_api::{model::HelloRequest, HelloServiceClient};
use showcase_client::HttpClient;
use structured_logger::{async_json::new_writer, Builder};
use tower::{ServiceBuilder, ServiceExt as _};
use tower_http::ServiceBuilderExt as _;
use tower_http_client::adapters::reqwest::HttpClientLayer;

fn make_client(client: reqwest::Client) -> HttpClient {
    let service = ServiceBuilder::new()
        // Add some layers.
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        // Make client compatible with the `tower-http` layers.
        .layer(HttpClientLayer)
        .service(client)
        .map_err(eyre::Error::from);
    tower_http_client::util::BoxCloneSyncService::new(service)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let inner_client = make_client(reqwest::Client::new());
    let server_address = format!("http://localhost:{}", showcase_api::DEFAULT_SERVER_PORT);

    let mut hello_client = showcase_client::HttpClientWithUrl::new(&server_address, inner_client);

    info!(
        server_address;
        "Sending simple hello request"
    );

    let response = hello_client
        .say_hello(HelloRequest {
            name: "Alice".to_string(),
        })
        .await?;
    info!(
        response:serde;
        "Received response"
    );

    Ok(())
}
