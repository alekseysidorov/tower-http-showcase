use axum::{
    Router,
    routing::{get, post},
};

use crate::state::SharedAppState;

pub fn make_router(state: SharedAppState) -> Router {
    Router::new()
        .route("/health", get(routes::health_check))
        .route("/hello", get(routes::hello_world))
        .route("/render", post(routes::render_tile))
        .with_state(state)
}

mod routes {
    use axum::{Json, extract::State, http::StatusCode};
    use fractal_core::{TileSpec, render_tile as render};
    use showcase_api::model::{
        HelloRequest, HelloResponse, RenderTileRequest, RenderTileResponse, RuntimeKind,
    };
    use std::time::Instant;

    use crate::state::{HelloService, SharedAppState};

    pub async fn health_check() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    pub async fn hello_world(
        State(state): State<SharedAppState>,
        Json(request): Json<HelloRequest>,
    ) -> Result<Json<HelloResponse>, StatusCode> {
        let message = state.hello_service().say_hello(&request.name).await;
        Ok(Json(HelloResponse { message }))
    }

    pub async fn render_tile(
        State(state): State<SharedAppState>,
        Json(request): Json<RenderTileRequest>,
    ) -> Result<Json<RenderTileResponse>, StatusCode> {
        let worker_delay = state.worker_delay();
        tokio::time::sleep(worker_delay).await;

        let started_at = Instant::now();
        let tile = render(&TileSpec {
            region: request.region,
            width: request.width,
            height: request.height,
            max_iterations: request.max_iterations,
        })
        .map_err(|_| StatusCode::BAD_REQUEST)?;
        let render_duration_ms = started_at.elapsed().as_secs_f64() * 1_000.0;

        Ok(Json(RenderTileResponse {
            tile_id: request.tile_id,
            worker_id: state.worker_id().to_owned(),
            runtime: RuntimeKind::Tokio,
            width: tile.width,
            height: tile.height,
            pixels: tile.pixels,
            render_duration_ms,
            artificial_delay_ms: worker_delay.as_secs_f64() * 1_000.0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use showcase_api::model::{RenderTileResponse, RuntimeKind};
    use tower::ServiceExt as _;

    use crate::{delay_iter::DelayIter, state::AppState};

    use super::make_router;

    #[tokio::test]
    async fn render_route_returns_a_valid_tile_response() {
        let state = AppState::with_worker(
            DelayIter::new(0, Duration::from_millis(1)..Duration::from_millis(2)),
            "integration-worker",
            Duration::ZERO,
        )
        .into();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"tile_id":7,"region":{"min_re":0.0,"max_re":0.0,"min_im":0.0,"max_im":0.0},"width":1,"height":1,"max_iterations":32}"#,
            ))
            .unwrap();

        let response = make_router(state).oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tile: RenderTileResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(tile.tile_id, 7);
        assert_eq!(tile.worker_id, "integration-worker");
        assert_eq!(tile.runtime, RuntimeKind::Tokio);
        assert_eq!((tile.width, tile.height), (1, 1));
        assert_eq!(tile.pixels, [0, 0, 0]);
    }
}
