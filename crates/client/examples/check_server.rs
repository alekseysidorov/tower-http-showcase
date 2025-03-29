use http::{header::USER_AGENT, HeaderValue};
use log::info;
use showcase_api::{model::HelloRequest, HelloService};
use showcase_client::HttpClient;
use structured_logger::{async_json::new_writer, Builder};
use tower::{ServiceBuilder, ServiceExt as _};
use tower_http::ServiceBuilderExt as _;
use tower_http_client::adapters::reqwest::HttpClientLayer;

fn make_client(client: reqwest::Client, node_address: String) -> HttpClient {
    let service = ServiceBuilder::new()
        // Add some layers.
        .map_request(move |mut request: http::Request<_>| {
            // Add node address to the request URI, since the underlying client relies on it.
            *request.uri_mut() = [&node_address, request.uri().path()]
                .concat()
                .parse()
                .unwrap();

            request
        })
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        // Make client compatible with the `tower-http` layers.
        .layer(HttpClientLayer)
        .service(client)
        .map_err(eyre::Error::from);
    HttpClient::new(tower::util::BoxCloneSyncService::new(service))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let server_address = format!("http://localhost:{}", showcase_api::DEFAULT_SERVER_PORT);
    for node_id in 0..16 {
        let node_address = format!("{server_address}/node/{node_id}");
        let mut hello_client = make_client(reqwest::Client::new(), node_address.clone());

        info!(
            node_address;
            "Sending simple hello request"
        );

        let response = hello_client
            .say_hello(HelloRequest {
                name: "Alice".to_string(),
            })
            .await?;
        info!(
            response:serde, node_id;
            "Received response"
        );
    }

    Ok(())
}
