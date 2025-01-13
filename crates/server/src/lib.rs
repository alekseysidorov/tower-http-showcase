use std::{convert::Infallible, time::Duration};

use axum::{
    http::{Response, StatusCode},
    BoxError, Router,
};
use serde::Serialize;
use tower::ServiceBuilder;

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

pub fn set_middlewares(router: Router) -> Router {
    router.layer(ServiceBuilder::new().map_future(|future| async move {
        let time = tokio::time::Instant::now();
        let response: Response<_> = future.await?;
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
    }))
}
