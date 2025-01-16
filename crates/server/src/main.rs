use std::time::Duration;

use axum::{error_handling::HandleErrorLayer, http::StatusCode, BoxError, Router};
use log::info;
use showcase_server::{
    config::AppConfig, delay_iter::DelayIter, http::make_router, middlewares::attach_middlewares,
    state::AppState,
};
use structured_logger::{async_json::new_writer, Builder};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let config = AppConfig::default();
    let service = {
        let mut router = Router::new();

        for node_id in 0..16 {
            let delay_iter = DelayIter::new(
                node_id,
                config.response_delays.min..config.response_delays.max,
            );
            let state = AppState::new(delay_iter);
            router = router.nest(
                &format!("/node/{node_id}"),
                make_router(state.into()).layer(
                    ServiceBuilder::new()
                        .layer(HandleErrorLayer::new(|err: BoxError| async move {
                            (StatusCode::INTERNAL_SERVER_ERROR, format!("`{err}"))
                        }))
                        .buffer(1024)
                        .rate_limit(250, Duration::from_secs(1)),
                ),
            )
        }

        attach_middlewares(router)
    };

    let address = format!("0.0.0.0:{}", showcase_api::DEFAULT_SERVER_PORT);
    let listener = TcpListener::bind(address).await?;

    info!(
        server_address:? = listener.local_addr();
        "Starting server"
    );

    axum::serve(listener, service).await?;

    Ok(())
}
