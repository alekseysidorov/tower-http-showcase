use axum::{routing::get, Router};
use log::info;
use showcase_common::model::HelloMessage;
use tokio::net::TcpListener;

async fn hello_world() -> axum::Json<HelloMessage> {
    axum::Json(HelloMessage {
        message: "Hello, World!".to_string(),
    })
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let router = Router::new().route("/hello", get(hello_world));

    let address = format!("0.0.0.0:{}", showcase_common::DEFAULT_SERVER_PORT);
    let listener = TcpListener::bind(address).await?;

    info!(
        server_address:? = listener.local_addr();
        "Starting server"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
