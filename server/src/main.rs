use axum::{response::IntoResponse, routing::get, Router};
use log::info;
use tokio::net::TcpListener;

async fn hello_world() -> impl IntoResponse {
    axum::Json("Hello, world!")
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let router = Router::new().route("/hello", get(hello_world));

    let address = format!("0.0.0.0:{}", showcase_common::DEFAULT_SERVER_PORT);
    let listener = TcpListener::bind(address).await?;

    info!(
        server_address:? = listener.local_addr();
        "Starting server..."
    );

    axum::serve(listener, router).await?;

    Ok(())
}
