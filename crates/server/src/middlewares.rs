use std::convert::Infallible;

use axum::{Router, extract::MatchedPath};
use context_logger::{ContextValue, FutureExt, LogContext};
use headers::HeaderMapExt as _;
use serde::Serialize;
use tower::{Service, ServiceBuilder, service_fn};

#[derive(Debug, Serialize)]
struct ResponseInfo {
    duration_ms: f64,
    status_code: u16,
}

#[derive(Debug, Serialize)]
struct RequestInfo {
    user_agent: Option<String>,
    matched_path: Option<String>,
}

pub fn attach_middlewares(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new().layer_fn(|mut service: axum::routing::Route| {
            service_fn(move |req: axum::extract::Request| {
                let user_agent = req
                    .headers()
                    .typed_get::<headers::UserAgent>()
                    .map(|x| x.to_string());
                let matched_path = req
                    .extensions()
                    .get::<MatchedPath>()
                    .map(|x| x.as_str().to_owned());

                let context = LogContext::new().record(
                    "request",
                    ContextValue::serde(RequestInfo {
                        user_agent,
                        matched_path,
                    }),
                );

                let fut = service.call(req);
                async move {
                    let time = tokio::time::Instant::now();
                    let response = fut.await?;
                    let elapsed = time.elapsed();

                    {
                        let response = ResponseInfo {
                            duration_ms: elapsed.as_secs_f64() * 1000.0,
                            status_code: response.status().as_u16(),
                        };

                        log::info!(
                            response:serde;
                            "Request handled"
                        );
                    }

                    Ok::<_, Infallible>(response)
                }
                .in_log_context(context)
            })
        }),
    )
}
