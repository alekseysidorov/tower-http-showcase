use axum::{Router, routing::get};

use crate::state::SharedAppState;

pub fn make_router(state: SharedAppState) -> Router {
    Router::new()
        .route("/hello", get(routes::hello_world))
        .with_state(state)
}

mod routes {
    use axum::{Json, extract::State, http::StatusCode};
    use showcase_api::model::{HelloRequest, HelloResponse};

    use crate::state::{HelloService, SharedAppState};

    pub async fn hello_world(
        State(state): State<SharedAppState>,
        Json(request): Json<HelloRequest>,
    ) -> Result<Json<HelloResponse>, StatusCode> {
        let message = state.hello_service().say_hello(&request.name).await;
        Ok(Json(HelloResponse { message }))
    }
}
