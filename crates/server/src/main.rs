use std::time::Duration;

use axum::{BoxError, error_handling::HandleErrorLayer, http::StatusCode};
use log::info;
use showcase_server::{
    config::AppConfig, delay_iter::DelayIter, http::make_router, middlewares::attach_middlewares,
    state::AppState,
};
use structured_logger::{Builder, async_json::new_writer};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let config = AppConfig::from_env()?;
    let service = {
        let worker_state = AppState::with_worker(
            DelayIter::new(0, config.response_delays.min..config.response_delays.max),
            config.worker_id,
            config.worker_delay,
        )
        .into();
        let mut router = make_router(worker_state);

        for node_id in 0..config.nodes_count {
            let delay_iter = DelayIter::new(
                node_id.into(),
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
                        .rate_limit(500, Duration::from_secs(1)),
                ),
            )
        }

        attach_middlewares(router)
    };

    let listener = TcpListener::bind(config.listen_addr).await?;

    info!(
        server_address:? = listener.local_addr();
        "Starting server"
    );

    axum::serve(listener, service).await?;

    Ok(())
}
