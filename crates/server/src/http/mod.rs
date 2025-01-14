use axum::{routing::get, Router};

use crate::state::SharedAppState;

pub fn make_router(state: SharedAppState) -> Router {
    Router::new()
        .route("/{node}/hello", get(routes::hello_world))
        .with_state(state)
}

mod routes {
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        Json,
    };
    use showcase_api::model::{HelloRequest, HelloResponse};

    use crate::state::{HelloService, SharedAppState};

    pub async fn hello_world(
        State(state): State<SharedAppState>,
        Path(node_id): Path<u16>,
        Json(request): Json<HelloRequest>,
    ) -> Result<Json<HelloResponse>, StatusCode> {
        let service = state
            .hello_service(node_id)
            .ok_or(StatusCode::BAD_REQUEST)?;

        let message = service.say_hello(&request.name).await;
        Ok(Json(HelloResponse { message }))
    }
}
