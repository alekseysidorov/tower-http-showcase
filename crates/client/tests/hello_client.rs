use showcase_api::{
    HelloService,
    model::{HelloRequest, HelloResponse},
};
use showcase_client::BoxedHttpClient;
use tower::ServiceBuilder;
use tower_reqwest::HttpClientLayer;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};

#[tokio::test]
async fn hello_client_sends_json_and_deserializes_response() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/node/7/hello"))
        .and(body_json(serde_json::json!({ "name": "Ada" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(HelloResponse {
            message: "Hello, Ada!".to_owned(),
        }))
        .expect(1)
        .mount(&mock_server)
        .await;

    let service = ServiceBuilder::new()
        .layer(HttpClientLayer)
        .service(reqwest::Client::new());
    let origin = format!("{}/node/7", mock_server.uri()).parse().unwrap();
    let mut client = BoxedHttpClient::with_origin(service, origin).unwrap();

    let response = client
        .say_hello(HelloRequest {
            name: "Ada".to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(response.message, "Hello, Ada!");
}
