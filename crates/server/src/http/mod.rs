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

    use crate::{service::HelloService as _, state::SharedAppState};

    pub async fn hello_world(
        State(state): State<SharedAppState>,
        Json(request): Json<HelloRequest>,
    ) -> Json<HelloResponse> {
        let message = state.say_hello(request.name).await;
        Json(HelloResponse { message })
    }
}
