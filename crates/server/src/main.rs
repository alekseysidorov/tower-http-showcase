use log::info;
use showcase_server::config::AppConfig;
use structured_logger::{async_json::new_writer, Builder};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let state = showcase_server::state::AppState::new(AppConfig::default()).into();
    let router = showcase_server::set_middlewares(showcase_server::http::make_router(state));

    let address = format!("0.0.0.0:{}", showcase_api::DEFAULT_SERVER_PORT);
    let listener = TcpListener::bind(address).await?;

    info!(
        server_address:? = listener.local_addr();
        "Starting server"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
