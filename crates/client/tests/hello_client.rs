use showcase_api::model::{HelloRequest, HelloResponse};
use showcase_api::{HelloError, HelloService};
use showcase_client::{HelloClient, into_tower_http_client, with_origin};
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

    let service = into_tower_http_client(reqwest::Client::new());
    let origin = format!("{}/node/7", mock_server.uri()).parse().unwrap();
    let mut client = HelloClient::new(with_origin(service, origin).unwrap());

    let response = client
        .say_hello(HelloRequest {
            name: "Ada".to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(response.message, "Hello, Ada!");
}

#[tokio::test]
async fn http_api_errors_are_distinct_from_transport_errors() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let origin = mock_server.uri().parse().unwrap();
    let service = into_tower_http_client(reqwest::Client::new());
    let mut client = HelloClient::new(with_origin(service, origin).unwrap());
    let error = client
        .say_hello(HelloRequest { name: "Ada".into() })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        HelloError::Api(http::StatusCode::SERVICE_UNAVAILABLE)
    ));
}

#[tokio::test]
async fn stopped_server_is_a_transport_error() {
    let origin = "http://127.0.0.1:1".parse().unwrap();
    let service = into_tower_http_client(reqwest::Client::new());
    let mut client = HelloClient::new(with_origin(service, origin).unwrap());
    let error = client
        .say_hello(HelloRequest { name: "Ada".into() })
        .await
        .unwrap_err();

    assert!(matches!(error, HelloError::Transport(_)));
}

#[tokio::test]
async fn malformed_success_response_is_a_protocol_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&mock_server)
        .await;

    let origin = mock_server.uri().parse().unwrap();
    let service = into_tower_http_client(reqwest::Client::new());
    let mut client = HelloClient::new(with_origin(service, origin).unwrap());
    let error = client
        .say_hello(HelloRequest { name: "Ada".into() })
        .await
        .unwrap_err();

    assert!(matches!(error, HelloError::Protocol(_)));
}
