use showcase_api::{
    HelloService,
    model::{HelloRequest, HelloResponse},
};
use tower::{BoxError, ServiceBuilder};
use tower_http_client::rewrite_uri::RewriteUriLayer;
use tower_reqwest::HttpClientLayer;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};

#[tokio::test]
async fn hello_client_sends_json_and_deserializes_response() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(body_json(serde_json::json!({ "name": "Ada" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(HelloResponse {
            message: "Hello, Ada!".to_owned(),
        }))
        .expect(1)
        .mount(&mock_server)
        .await;

    let base_uri = mock_server.uri();
    let service = ServiceBuilder::new()
        .layer(RewriteUriLayer::new(move |uri: &http::Uri| {
            let path_and_query = uri.path_and_query().map_or("/", |value| value.as_str());
            format!("{base_uri}{path_and_query}")
                .parse::<http::Uri>()
                .map_err(BoxError::from)
        }))
        .map_err(BoxError::from)
        .layer(HttpClientLayer)
        .service(reqwest::Client::new());
    let mut client = showcase_client::HelloClient::new(service);

    let response = client
        .say_hello(HelloRequest {
            name: "Ada".to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(response.message, "Hello, Ada!");
}
