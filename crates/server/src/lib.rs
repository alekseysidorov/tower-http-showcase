use std::convert::Infallible;

use axum::Router;
use headers::HeaderMapExt as _;
use serde::Serialize;
use tower::{service_fn, Service, ServiceBuilder};

pub mod config;
pub mod delay_iter;
pub mod http;
pub mod metrics;
pub mod state;

#[derive(Debug, Serialize)]
struct ResponseInfo {
    duration_ms: f64,
    status_code: u16,
}

#[derive(Debug, Serialize)]
struct RequestInfo {
    user_agent: Option<String>,
}

pub fn attach_middlewares(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new().layer_fn(|mut service: axum::routing::Route| {
            service_fn(move |req: axum::extract::Request| {
                let user_agent = req
                    .headers()
                    .typed_get::<headers::UserAgent>()
                    .map(|x| x.to_string());

                let fut = service.call(req);
                async move {
                    let time = tokio::time::Instant::now();
                    let response = fut.await?;
                    let elapsed = time.elapsed();

                    {
                        let request = RequestInfo { user_agent };
                        let response = ResponseInfo {
                            duration_ms: elapsed.as_secs_f64() * 1000.0,
                            status_code: response.status().as_u16(),
                        };
                        log::info!(
                            request:serde, response:serde;
                            "Request handled"
                        );
                    }

                    Ok::<_, Infallible>(response)
                }
            })
        }),
    )
}
