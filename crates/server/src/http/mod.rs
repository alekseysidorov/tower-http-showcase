use axum::{routing::get, Router};

use crate::state::SharedAppState;

pub fn make_router(state: SharedAppState) -> Router {
    Router::new()
        .route("/hello", get(routes::hello_world))
        .with_state(state)
}

mod routes {
    use axum::{extract::State, Json};
    use showcase_api::model::{HelloRequest, HelloResponse};

    use crate::state::SharedAppState;

    pub async fn hello_world(
        State(_state): State<SharedAppState>,
        Json(request): Json<HelloRequest>,
    ) -> Json<HelloResponse> {
        Json(HelloResponse {
            message: format!("Hello, {}!", request.name),
        })
    }
}
