use std::borrow::Cow;
use std::time::Duration;

use axum::{BoxError, Router, error_handling::HandleErrorLayer, http::StatusCode};
use context_logger::ContextLogger;
use fastrace::{Span, collector::Config, local::LocalSpan, prelude::SpanContext};
use fastrace_opentelemetry::OpenTelemetryReporter;
use fastrace_tower::FastraceServerLayer;
use log::{LevelFilter, info};
use opentelemetry_otlp::WithExportConfig as _;
use showcase_api::NODES_COUNT;
use showcase_server::{
    config::AppConfig, delay_iter::DelayIter, http::make_router, middlewares::attach_middlewares,
    state::AppState,
};
use structured_logger::{Builder, async_json::new_writer};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let level = LevelFilter::Info;
    ContextLogger::new(
        Builder::with_level(level.as_str())
            .with_target_writer("*", new_writer(tokio::io::stdout()))
            .build(),
    )
    .init(level);

    // Initialize fastrace reporter.
    let reporter = OpenTelemetryReporter::new(
        opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint("http://127.0.0.1:4317".to_string())
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_timeout(opentelemetry_otlp::OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
            .build()
            .expect("initialize oltp exporter"),
        Cow::Owned(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new(
                    "service.name",
                    "tower-http-server",
                )])
                .build(),
        ),
        opentelemetry::InstrumentationScope::builder("server")
            .with_version(env!("CARGO_PKG_VERSION"))
            .build(),
    );

    // let reporter = ConsoleReporter;
    fastrace::set_reporter(reporter, Config::default());

    {
        // Start tracing
        let root = Span::root("root", SpanContext::random());

        let _g = root.set_local_parent();
        let _span = LocalSpan::enter_with_local_parent("a span")
            .with_property(|| ("a property", "a value"));
    }
    fastrace::flush();

    let config = AppConfig::default();
    let service = {
        let mut router = Router::new();

        for node_id in 0..NODES_COUNT {
            let delay_iter = DelayIter::new(
                node_id,
                config.response_delays.min..config.response_delays.max,
            );
            let state = AppState::new(delay_iter);
            router = router.nest(
                &format!("/node/{node_id}"),
                make_router(state.into()).layer(
                    ServiceBuilder::new()
                        .layer(FastraceServerLayer)
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

    let address = format!("0.0.0.0:{}", showcase_api::DEFAULT_SERVER_PORT);
    let listener = TcpListener::bind(address).await?;

    info!(
        server_address:? = listener.local_addr();
        "Starting server"
    );

    axum::serve(listener, service).await?;

    fastrace::flush();
    Ok(())
}
