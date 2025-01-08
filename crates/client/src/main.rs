use http::{header::USER_AGENT, HeaderValue};
use log::info;
use showcase_common::model::HelloMessage;
use tower::{ServiceBuilder, ServiceExt as _};
use tower_http::ServiceBuilderExt as _;
use tower_http_client::{adapters::reqwest::HttpClientLayer, ResponseExt as _, ServiceExt as _};

/// Implementation agnostic HTTP client.
type HttpClient = tower::util::BoxCloneService<
    http::Request<reqwest::Body>,
    http::Response<reqwest::Body>,
    eyre::Error,
>;

fn make_client(client: reqwest::Client) -> HttpClient {
    ServiceBuilder::new()
        // Add some layers.
        .override_request_header(USER_AGENT, HeaderValue::from_static("tower-http-client"))
        // Make client compatible with the `tower-http` layers.
        .layer(HttpClientLayer)
        .service(client)
        .map_err(eyre::Error::from)
        .boxed_clone()
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let mut client = make_client(reqwest::Client::new());
    let server_address = format!("http://localhost:{}", showcase_common::DEFAULT_SERVER_PORT);

    info!(
        server_address;
        "Sending simple hello request"
    );

    let response = client
        .get(format!("{server_address}/hello"))
        .send()?
        .await?;

    let body = response.body_reader().json::<HelloMessage>().await?;
    info!(
        body:serde;
        "Received response"
    );

    Ok(())
}
