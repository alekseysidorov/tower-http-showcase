use axum::{
    Router,
    routing::{get, post},
};

use crate::state::SharedAppState;

pub fn make_router(state: SharedAppState) -> Router {
    Router::new()
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
